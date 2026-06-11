use openh264::encoder::{Encoder, EncoderConfig};
use openh264::formats::YUVSource;
use openh264::OpenH264API;
use crate::{EncoderError, EncoderInput, EncodedFrame, VideoEncoder};

pub struct OpenH264Encoder {
    encoder: Encoder,
    force_keyframe: bool,
}

impl OpenH264Encoder {
    pub fn new(
        _width: u32,
        _height: u32,
        _fps: u32,
        bitrate_bps: u32,
    ) -> Result<Self, EncoderError> {
        let api = OpenH264API::from_source();
        let config = EncoderConfig::new()
            .set_bitrate_bps(bitrate_bps)
            .enable_skip_frame(false);
        let encoder = Encoder::with_api_config(api, config)
            .map_err(|e| EncoderError::Init(e.to_string()))?;
        Ok(Self { encoder, force_keyframe: false })
    }
}

impl VideoEncoder for OpenH264Encoder {
    fn encode(&mut self, input: EncoderInput) -> Result<Option<EncodedFrame>, EncoderError> {
        let w = input.width as usize;
        let h = input.height as usize;
        let y_size = w * h;
        let uv_size = w / 2 * h / 2;

        if input.yuv_data.len() < y_size + 2 * uv_size {
            return Err(EncoderError::Encode("Buffer YUV trop court".into()));
        }

        let y = &input.yuv_data[..y_size];
        let u = &input.yuv_data[y_size..y_size + uv_size];
        let v = &input.yuv_data[y_size + uv_size..];

        struct I420Buf<'a> {
            y: &'a [u8],
            u: &'a [u8],
            v: &'a [u8],
            w: usize,
            h: usize,
        }

        impl<'a> YUVSource for I420Buf<'a> {
            fn dimensions(&self) -> (usize, usize) {
                (self.w, self.h)
            }
            fn strides(&self) -> (usize, usize, usize) {
                (self.w, self.w / 2, self.w / 2)
            }
            fn y(&self) -> &[u8] { self.y }
            fn u(&self) -> &[u8] { self.u }
            fn v(&self) -> &[u8] { self.v }
        }

        let src = I420Buf { y, u, v, w, h };
        let bitstream = self
            .encoder
            .encode(&src)
            .map_err(|e| EncoderError::Encode(e.to_string()))?;

        let data: Vec<u8> = bitstream.to_vec();
        if data.is_empty() {
            return Ok(None);
        }

        let keyframe = self.force_keyframe;
        self.force_keyframe = false;

        Ok(Some(EncodedFrame { data, keyframe, timestamp_ms: input.timestamp_ms }))
    }

    fn request_keyframe(&mut self) {
        self.force_keyframe = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bgra_to_yuv420, EncoderInput, VideoEncoder};

    fn make_test_input(width: u32, height: u32, ts: u64) -> EncoderInput {
        let bgra = vec![128u8; (width * height * 4) as usize];
        EncoderInput {
            yuv_data: bgra_to_yuv420(&bgra, width, height),
            width,
            height,
            timestamp_ms: ts,
        }
    }

    #[test]
    fn first_frame_produces_output() {
        let mut enc = OpenH264Encoder::new(640, 360, 30, 1_000_000).unwrap();
        let input = make_test_input(640, 360, 0);
        let result = enc.encode(input).unwrap();
        let frame = result.expect("La première frame doit produire des données");
        assert!(!frame.data.is_empty());
    }

    #[test]
    fn encode_multiple_frames_without_error() {
        let mut enc = OpenH264Encoder::new(640, 360, 30, 1_000_000).unwrap();
        for i in 0..5 {
            let input = make_test_input(640, 360, i * 33);
            enc.encode(input).expect("L'encodage ne doit pas échouer");
        }
    }
}
