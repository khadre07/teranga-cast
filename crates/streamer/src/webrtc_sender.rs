use std::sync::Arc;
use std::time::Duration;
use bytes::Bytes;
use thiserror::Error;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::media::Sample;

#[derive(Debug, Error)]
pub enum WebRtcError {
    #[error("WebRTC : {0}")]
    Internal(String),
}

pub struct WebRtcSender {
    peer_connection: Arc<RTCPeerConnection>,
    video_track: Arc<TrackLocalStaticSample>,
    local_sdp: String,
}

impl WebRtcSender {
    pub async fn new() -> Result<Self, WebRtcError> {
        let mut media_engine = MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        let api = APIBuilder::new().with_media_engine(media_engine).build();

        // Pas de STUN : réseau 100% local
        let config = RTCConfiguration::default();

        let peer_connection = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| WebRtcError::Internal(e.to_string()))?,
        );

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                        .to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "terangacast".to_owned(),
        ));

        peer_connection
            .add_track(video_track.clone())
            .await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        peer_connection
            .set_local_description(offer)
            .await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        tokio::select! {
            _ = gather_complete.recv() => {}
            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
        }

        let local_desc = peer_connection
            .local_description()
            .await
            .ok_or_else(|| WebRtcError::Internal("Pas de description locale".into()))?;

        let local_sdp = serde_json::to_string(&local_desc)
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        Ok(Self { peer_connection, video_track, local_sdp })
    }

    pub fn local_sdp(&self) -> &str {
        &self.local_sdp
    }

    pub async fn set_remote_answer(&self, answer_json: &str) -> Result<(), WebRtcError> {
        let answer: RTCSessionDescription = serde_json::from_str(answer_json)
            .map_err(|e| WebRtcError::Internal(format!("JSON invalide : {}", e)))?;
        self.peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| WebRtcError::Internal(e.to_string()))
    }

    pub async fn send_frame(&self, data: Vec<u8>, duration_ms: u64) -> Result<(), WebRtcError> {
        self.video_track
            .write_sample(&Sample {
                data: Bytes::from(data),
                duration: Duration::from_millis(duration_ms),
                ..Default::default()
            })
            .await
            .map_err(|e| WebRtcError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_sender_generates_sdp_offer() {
        let sender = WebRtcSender::new().await.expect("Création sender échouée");
        let offer = sender.local_sdp();
        assert!(!offer.is_empty(), "Le SDP offer ne doit pas être vide");
        assert!(offer.contains("m=video"), "Le SDP doit contenir une piste vidéo");
    }
}
