use serde::{Deserialize, Serialize};
use tauri::State;
use terangacast_encoder::OpenH264Encoder;
use terangacast_streamer::{QualityConfig, StreamManager};

#[cfg(target_os = "windows")]
use terangacast_capture::ScapCapture;

use crate::state::AppState;

#[derive(Serialize)]
pub struct StreamInfoDto {
    pub url: String,
    pub local_ip: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
}

#[tauri::command]
pub async fn start_stream(
    state: State<'_, AppState>,
    quality: QualityLevel,
) -> Result<StreamInfoDto, String> {
    let mut lock = state.stream.lock().await;
    if lock.is_some() {
        return Err("Un stream est déjà en cours".into());
    }

    let quality_config = match quality {
        QualityLevel::Low => QualityConfig::low(),
        QualityLevel::Medium => QualityConfig::medium(),
        QualityLevel::High => QualityConfig::high(),
    };

    let fps = quality_config.fps;
    let bitrate = quality_config.bitrate_bps;
    let width = quality_config.width;
    let height = quality_config.height;

    #[cfg(target_os = "windows")]
    let capture = Box::new(ScapCapture::new(fps).map_err(|e| e.to_string())?);

    #[cfg(not(target_os = "windows"))]
    let capture: Box<dyn terangacast_capture::ScreenCapture> = {
        return Err("La capture d'écran nécessite Windows".into());
    };

    let encoder = Box::new(
        OpenH264Encoder::new(width, height, fps, bitrate).map_err(|e| e.to_string())?,
    );

    let (manager, info) = StreamManager::start(capture, encoder, quality_config, 8765)
        .await
        .map_err(|e| e.to_string())?;

    let dto = StreamInfoDto { url: info.url, local_ip: info.local_ip, port: info.port };
    *lock = Some(manager);
    Ok(dto)
}

#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>) -> Result<(), String> {
    let mut lock = state.stream.lock().await;
    if let Some(manager) = lock.take() {
        manager.stop();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> bool {
    state.stream.lock().await.is_some()
}
