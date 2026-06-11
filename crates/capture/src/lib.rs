use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Initialisation échouée : {0}")]
    Init(String),
    #[error("Capture de frame échouée : {0}")]
    Frame(String),
}

#[derive(Debug, Clone)]
pub struct RawFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub trait ScreenCapture: Send {
    fn start(&mut self) -> Result<(), CaptureError>;
    fn next_frame(&mut self) -> Result<RawFrame, CaptureError>;
    fn stop(&mut self);
    fn resolution(&self) -> (u32, u32);
}

#[cfg(target_os = "windows")]
pub mod scap_capture;
#[cfg(target_os = "windows")]
pub use scap_capture::ScapCapture;

#[cfg(test)]
pub struct MockCapture {
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
}

#[cfg(test)]
impl ScreenCapture for MockCapture {
    fn start(&mut self) -> Result<(), CaptureError> {
        Ok(())
    }

    fn next_frame(&mut self) -> Result<RawFrame, CaptureError> {
        self.frame_count += 1;
        let pixel_count = (self.width * self.height) as usize;
        let mut data = vec![0u8; pixel_count * 4];
        for px in data.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 255;
            px[3] = 255;
        }
        Ok(RawFrame { data, width: self.width, height: self.height })
    }

    fn stop(&mut self) {}

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_frame_with_correct_dimensions() {
        let mut cap = MockCapture { width: 1920, height: 1080, frame_count: 0 };
        cap.start().unwrap();
        let frame = cap.next_frame().unwrap();
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
        assert_eq!(frame.data.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn mock_increments_frame_count() {
        let mut cap = MockCapture { width: 640, height: 480, frame_count: 0 };
        cap.start().unwrap();
        cap.next_frame().unwrap();
        cap.next_frame().unwrap();
        assert_eq!(cap.frame_count, 2);
    }
}
