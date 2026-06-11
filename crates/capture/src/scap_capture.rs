use scap::capturer::{Capturer, Options, Resolution};
use scap::frame::{Frame, FrameType};
use crate::{CaptureError, RawFrame, ScreenCapture};

pub struct ScapCapture {
    capturer: Capturer,
    width: u32,
    height: u32,
}

impl ScapCapture {
    pub fn new(fps: u32) -> Result<Self, CaptureError> {
        if !scap::is_supported() {
            return Err(CaptureError::Init(
                "Capture d'écran non supportée sur ce système".into(),
            ));
        }

        let options = Options {
            fps,
            target: None,
            show_cursor: true,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::Captured,
            source_rect: None,
            ..Default::default()
        };

        let capturer = Capturer::build(options)
            .map_err(|e| CaptureError::Init(e.to_string()))?;

        let [width, height] = capturer.get_output_frame_size();

        Ok(Self { capturer, width, height })
    }
}

impl ScreenCapture for ScapCapture {
    fn start(&mut self) -> Result<(), CaptureError> {
        self.capturer.start_capture();
        Ok(())
    }

    fn next_frame(&mut self) -> Result<RawFrame, CaptureError> {
        match self.capturer.get_next_frame() {
            Ok(Frame::BGRA(f)) => Ok(RawFrame {
                data: f.data,
                width: f.width,
                height: f.height,
            }),
            Ok(_) => Err(CaptureError::Frame("Format de frame inattendu".into())),
            Err(e) => Err(CaptureError::Frame(e.to_string())),
        }
    }

    fn stop(&mut self) {
        self.capturer.stop_capture();
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
