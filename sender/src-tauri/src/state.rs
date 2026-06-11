use tokio::sync::Mutex;
use terangacast_streamer::StreamManager;

pub struct AppState {
    pub stream: Mutex<Option<StreamManager>>,
}

impl AppState {
    pub fn new() -> Self {
        Self { stream: Mutex::new(None) }
    }
}
