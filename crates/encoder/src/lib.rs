pub mod yuv;
pub use yuv::bgra_to_yuv420;

pub mod openh264_encoder;
pub use openh264_encoder::OpenH264Encoder;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Initialisation encodeur : {0}")]
    Init(String),
    #[error("Encodage : {0}")]
    Encode(String),
}

pub struct EncoderInput {
    pub yuv_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub keyframe: bool,
    pub timestamp_ms: u64,
}

pub trait VideoEncoder: Send {
    fn encode(&mut self, input: EncoderInput) -> Result<Option<EncodedFrame>, EncoderError>;
    fn request_keyframe(&mut self);
}
