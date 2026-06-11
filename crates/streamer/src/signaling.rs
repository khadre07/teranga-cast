use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use crate::webrtc_sender::WebRtcSender;

pub struct SignalingState {
    pub sender: Arc<WebRtcSender>,
    pub answer_tx: Mutex<Option<oneshot::Sender<String>>>,
    pub answer_rx: Mutex<Option<oneshot::Receiver<String>>>,
}

impl SignalingState {
    pub fn new(sender: Arc<WebRtcSender>) -> Arc<Self> {
        let (tx, rx) = oneshot::channel();
        Arc::new(Self {
            sender,
            answer_tx: Mutex::new(Some(tx)),
            answer_rx: Mutex::new(Some(rx)),
        })
    }
}

const RECEIVER_HTML: &str = include_str!("../../../receiver/index.html");
const RECEIVER_JS: &str = include_str!("../../../receiver/receiver.js");

async fn serve_html() -> Html<&'static str> {
    Html(RECEIVER_HTML)
}

async fn serve_js() -> (StatusCode, [(axum::http::header::HeaderName, &'static str); 1], &'static str) {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        RECEIVER_JS,
    )
}

async fn get_offer(State(state): State<Arc<SignalingState>>) -> Json<Value> {
    let sdp_json: Value = serde_json::from_str(state.sender.local_sdp())
        .unwrap_or(Value::Null);
    Json(sdp_json)
}

async fn post_answer(
    State(state): State<Arc<SignalingState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let answer_str = body.to_string();
    let mut tx_guard = state.answer_tx.lock().await;
    if let Some(tx) = tx_guard.take() {
        let _ = tx.send(answer_str);
    }
    Json(serde_json::json!({ "ok": true }))
}

pub fn build_router(state: Arc<SignalingState>) -> Router {
    Router::new()
        .route("/", get(serve_html))
        .route("/receiver.js", get(serve_js))
        .route("/offer", get(get_offer))
        .route("/answer", post(post_answer))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn make_state() -> Arc<SignalingState> {
        let sender = Arc::new(WebRtcSender::new().await.expect("sender OK"));
        SignalingState::new(sender)
    }

    #[tokio::test]
    async fn get_offer_returns_200() {
        let state = make_state().await;
        let app = build_router(state);
        let req = Request::builder().uri("/offer").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn root_serves_html_page() {
        let state = make_state().await;
        let app = build_router(state);
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
