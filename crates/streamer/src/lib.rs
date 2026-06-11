pub mod signaling;
pub mod webrtc_sender;

use std::sync::Arc;
use terangacast_capture::ScreenCapture;
use terangacast_encoder::{bgra_to_yuv420, EncoderInput, VideoEncoder};
use thiserror::Error;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use signaling::SignalingState;
use webrtc_sender::WebRtcSender;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("WebRTC : {0}")]
    WebRtc(String),
    #[error("Réseau : {0}")]
    Network(String),
}

pub struct StreamInfo {
    pub url: String,
    pub local_ip: String,
    pub port: u16,
}

pub struct QualityConfig {
    pub fps: u32,
    pub bitrate_bps: u32,
    pub width: u32,
    pub height: u32,
}

impl QualityConfig {
    pub fn low() -> Self {
        Self { fps: 15, bitrate_bps: 800_000, width: 1280, height: 720 }
    }
    pub fn medium() -> Self {
        Self { fps: 24, bitrate_bps: 2_000_000, width: 1920, height: 1080 }
    }
    pub fn high() -> Self {
        Self { fps: 30, bitrate_bps: 4_000_000, width: 1920, height: 1080 }
    }
}

pub struct StreamManager {
    stop_tx: watch::Sender<bool>,
    _capture_task: JoinHandle<()>,
    _signal_task: JoinHandle<()>,
}

impl StreamManager {
    pub async fn start(
        mut capture: Box<dyn ScreenCapture>,
        mut encoder: Box<dyn VideoEncoder>,
        quality: QualityConfig,
        port: u16,
    ) -> Result<(Self, StreamInfo), StreamError> {
        let sender = Arc::new(
            WebRtcSender::new()
                .await
                .map_err(|e| StreamError::WebRtc(e.to_string()))?,
        );

        let sig_state = SignalingState::new(sender.clone());
        let router = signaling::build_router(sig_state.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|e| StreamError::Network(e.to_string()))?;

        let local_ip = local_ip_address().unwrap_or_else(|| "127.0.0.1".to_owned());
        let url = format!("http://{}:{}", local_ip, port);

        let signal_task = tokio::spawn(async move {
            axum::serve(listener, router).await.ok();
        });

        let (stop_tx, stop_rx) = watch::channel(false);
        let sender_clone = sender.clone();
        let frame_ms = 1000 / quality.fps as u64;

        capture.start().ok();

        let capture_task = tokio::spawn(async move {
            loop {
                if *stop_rx.borrow() {
                    break;
                }

                match capture.next_frame() {
                    Ok(frame) => {
                        let yuv = bgra_to_yuv420(&frame.data, frame.width, frame.height);
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        let input = EncoderInput {
                            yuv_data: yuv,
                            width: frame.width,
                            height: frame.height,
                            timestamp_ms: ts,
                        };

                        match encoder.encode(input) {
                            Ok(Some(encoded)) => {
                                if let Err(e) =
                                    sender_clone.send_frame(encoded.data, frame_ms).await
                                {
                                    warn!("Envoi frame échoué : {}", e);
                                }
                            }
                            Ok(None) => {}
                            Err(e) => warn!("Encodage échoué : {}", e),
                        }
                    }
                    Err(e) => warn!("Capture échouée : {}", e),
                }

                tokio::time::sleep(std::time::Duration::from_millis(frame_ms)).await;
            }
            capture.stop();
            info!("Pipeline de capture arrêtée");
        });

        let manager = Self {
            stop_tx,
            _capture_task: capture_task,
            _signal_task: signal_task,
        };

        Ok((manager, StreamInfo { url, local_ip, port }))
    }

    pub fn stop(&self) {
        let _ = self.stop_tx.send(true);
    }
}

fn local_ip_address() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
