/// Convertit un buffer BGRA (4 octets/pixel) en YUV420 planaire (I420).
/// Retourne Y (w*h) + U (w/2 * h/2) + V (w/2 * h/2) concaténés.
pub fn bgra_to_yuv420(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let y_size = w * h;
    let uv_size = (w / 2) * (h / 2);
    let mut yuv = vec![0u8; y_size + 2 * uv_size];

    let (y_plane, uv_planes) = yuv.split_at_mut(y_size);
    let (u_plane, v_plane) = uv_planes.split_at_mut(uv_size);

    for row in 0..h {
        for col in 0..w {
            let idx = (row * w + col) * 4;
            let b = data[idx] as f32;
            let g = data[idx + 1] as f32;
            let r = data[idx + 2] as f32;

            y_plane[row * w + col] =
                (0.257 * r + 0.504 * g + 0.098 * b + 16.0).clamp(0.0, 255.0) as u8;

            if row % 2 == 0 && col % 2 == 0 {
                let uv_idx = (row / 2) * (w / 2) + (col / 2);
                u_plane[uv_idx] =
                    (-(0.148 * r) - 0.291 * g + 0.439 * b + 128.0).clamp(0.0, 255.0) as u8;
                v_plane[uv_idx] =
                    (0.439 * r - 0.368 * g - 0.071 * b + 128.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    yuv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_length_correct_for_1080p() {
        let width = 1920u32;
        let height = 1080u32;
        let input = vec![0u8; (width * height * 4) as usize];
        let yuv = bgra_to_yuv420(&input, width, height);
        let expected = (width * height + width / 2 * height / 2 * 2) as usize;
        assert_eq!(yuv.len(), expected);
    }

    #[test]
    fn pure_red_pixel_has_expected_y() {
        let width = 2u32;
        let height = 2u32;
        // BGRA pur rouge : B=0, G=0, R=255, A=255
        let bgra = vec![0u8, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255];
        let yuv = bgra_to_yuv420(&bgra, width, height);
        // Y rouge ≈ 0.257*255 + 16 ≈ 82
        let y = yuv[0];
        assert!(y > 70 && y < 95, "Y rouge attendu ~82, obtenu {}", y);
    }

    #[test]
    fn pure_white_pixel_y_near_255() {
        let width = 2u32;
        let height = 2u32;
        let bgra = vec![255u8; 16];
        let yuv = bgra_to_yuv420(&bgra, width, height);
        let y = yuv[0];
        assert!(y > 220, "Y blanc attendu proche de 255, obtenu {}", y);
    }
}
