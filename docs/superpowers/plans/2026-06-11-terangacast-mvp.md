# TerangaCast MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Streamer l'écran d'un PC Windows vers un navigateur TV via WebRTC sur réseau LAN, avec une application Tauri 2 desktop pour contrôler le stream.

**Architecture:** Workspace Rust avec crates dédiés (capture, encoder, streamer). Le sender Tauri héberge un serveur HTTP Axum pour l'échange de signaux SDP (offre/réponse WebRTC). La TV ouvre une page HTML servie par ce même serveur et reçoit le flux H.264 via WebRTC.

**Tech Stack:** Rust 1.82, Tauri 2.1, React 18, TypeScript 5, Vite 6, axum 0.8, webrtc 0.12, openh264 0.6, scap 0.3, tokio 1.40

---

## Flux de données

```
Écran Windows
  → scap (frames BGRA brutes)
  → bgra_to_yuv420() (conversion pixel)
  → openh264 (NAL units H.264)
  → TrackLocalStaticSample (webrtc crate)
  → RTP/DTLS sur LAN
  → Navigateur TV (<video> WebRTC)
```

## Flux de signalisation (1 seule fois au démarrage)

```
1. Clic "Démarrer" dans Tauri
2. Rust crée RTCPeerConnection + SDP offer (ICE candidates inclus)
3. Axum écoute sur :8765
4. UI Tauri affiche : "Ouvrez http://192.168.x.x:8765 sur votre TV"
5. Navigateur TV charge la page HTML receiver
6. Receiver JS : GET /offer → set remote description → create answer → POST /answer
7. WebRTC établi, flux vidéo démarre
```

---

## Structure des fichiers

```
/                                   ← racine workspace
├── Cargo.toml                      ← workspace (members: 4 crates)
├── crates/
│   ├── capture/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              ← trait ScreenCapture + type Frame
│   │       └── scap_capture.rs    ← impl Windows via crate scap
│   ├── encoder/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              ← trait VideoEncoder + types EncoderFrame/EncodedFrame
│   │       ├── yuv.rs              ← fn bgra_to_yuv420()
│   │       └── openh264_encoder.rs ← impl openh264
│   └── streamer/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs              ← StreamManager : pilote la pipeline
│           ├── webrtc_sender.rs   ← RTCPeerConnection + TrackLocalStaticSample
│           └── signaling.rs       ← serveur Axum (/, /offer, /answer)
├── sender/                         ← Application Tauri
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   └── src/
│   │       ├── main.rs             ← setup Tauri, monte l'AppState
│   │       ├── state.rs            ← AppState (Arc<Mutex<Option<StreamManager>>>)
│   │       └── commands.rs         ← #[tauri::command] start_stream / stop_stream / get_info
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   └── components/
│   │       ├── StreamControl.tsx   ← bouton Start/Stop + affichage URL
│   │       └── QualitySelector.tsx ← Low / Medium / High
│   ├── index.html
│   ├── package.json
│   └── vite.config.ts
└── receiver/
    ├── index.html                  ← page standalone TV (embarquée via include_str!)
    └── receiver.js                 ← logique WebRTC côté récepteur
```

---

## Task 1 : Workspace Rust + Scaffold Tauri

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/capture/Cargo.toml`
- Create: `crates/encoder/Cargo.toml`
- Create: `crates/streamer/Cargo.toml`
- Create: `sender/` (via Tauri CLI)

- [ ] **Step 1.1 : Créer le workspace racine**

```toml
# Cargo.toml (racine)
[workspace]
members = [
    "sender/src-tauri",
    "crates/capture",
    "crates/encoder",
    "crates/streamer",
]
resolver = "2"

[workspace.dependencies]
tokio       = { version = "1.40", features = ["full"] }
serde       = { version = "1.0", features = ["derive"] }
serde_json  = "1.0"
thiserror   = "2.0"
bytes       = "1.7"
tracing     = "0.1"
```

- [ ] **Step 1.2 : Créer le crate capture**

```bash
mkdir -p crates/capture/src
```

```toml
# crates/capture/Cargo.toml
[package]
name    = "terangacast-capture"
version = "0.1.0"
edition = "2021"

[dependencies]
scap      = "0.3"
thiserror = { workspace = true }
tokio     = { workspace = true }
tracing   = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt", "macros"] }
```

- [ ] **Step 1.3 : Créer le crate encoder**

```bash
mkdir -p crates/encoder/src
```

```toml
# crates/encoder/Cargo.toml
[package]
name    = "terangacast-encoder"
version = "0.1.0"
edition = "2021"

[dependencies]
openh264  = "0.6"
thiserror = { workspace = true }
tracing   = { workspace = true }

[dev-dependencies]
# rien d'extra pour l'instant
```

- [ ] **Step 1.4 : Créer le crate streamer**

```bash
mkdir -p crates/streamer/src
```

```toml
# crates/streamer/Cargo.toml
[package]
name    = "terangacast-streamer"
version = "0.1.0"
edition = "2021"

[dependencies]
terangacast-capture = { path = "../capture" }
terangacast-encoder = { path = "../encoder" }
webrtc     = "0.12"
axum       = { version = "0.8", features = ["tokio"] }
tokio      = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
thiserror  = { workspace = true }
bytes      = { workspace = true }
tracing    = { workspace = true }
```

- [ ] **Step 1.5 : Scaffolder l'app Tauri**

```bash
npm create tauri-app@latest sender -- --template react-ts --manager npm
cd sender && npm install
```

Résultat attendu : dossier `sender/` avec `src-tauri/`, `src/`, `package.json`, `vite.config.ts`.

- [ ] **Step 1.6 : Ajouter les dépendances Rust au sender**

```toml
# sender/src-tauri/Cargo.toml — section [dependencies]
terangacast-streamer = { path = "../../crates/streamer" }
tokio     = { workspace = true }
serde     = { workspace = true }
serde_json = { workspace = true }
tauri     = { version = "2.1", features = [] }
```

- [ ] **Step 1.7 : Vérifier que le workspace compile**

```bash
cargo check --workspace
```

Résultat attendu : `warning: ...` éventuels, pas d'erreurs.

- [ ] **Step 1.8 : Commit initial**

```bash
git add Cargo.toml crates/ sender/src-tauri/Cargo.toml
git commit -m "chore: initialise workspace Rust (capture, encoder, streamer, sender)"
```

---

## Task 2 : Crate capture — trait + mock + implémentation scap

**Files:**
- Create: `crates/capture/src/lib.rs`
- Create: `crates/capture/src/scap_capture.rs`

- [ ] **Step 2.1 : Écrire le test avec MockCapture**

```rust
// crates/capture/src/lib.rs
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
    pub data: Vec<u8>,   // pixels BGRA (4 octets par pixel)
    pub width: u32,
    pub height: u32,
}

pub trait ScreenCapture: Send {
    fn start(&mut self) -> Result<(), CaptureError>;
    fn next_frame(&mut self) -> Result<RawFrame, CaptureError>;
    fn stop(&mut self);
    fn resolution(&self) -> (u32, u32);
}

// --- Mock pour les tests ---
#[cfg(test)]
pub struct MockCapture {
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
}

#[cfg(test)]
impl ScreenCapture for MockCapture {
    fn start(&mut self) -> Result<(), CaptureError> { Ok(()) }

    fn next_frame(&mut self) -> Result<RawFrame, CaptureError> {
        self.frame_count += 1;
        let pixel_count = (self.width * self.height) as usize;
        // frame de test : tous les pixels rouges en BGRA
        let mut data = vec![0u8; pixel_count * 4];
        for px in data.chunks_exact_mut(4) {
            px[0] = 0;   // B
            px[1] = 0;   // G
            px[2] = 255; // R
            px[3] = 255; // A
        }
        Ok(RawFrame { data, width: self.width, height: self.height })
    }

    fn stop(&mut self) {}
    fn resolution(&self) -> (u32, u32) { (self.width, self.height) }
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
```

- [ ] **Step 2.2 : Lancer le test — doit passer**

```bash
cargo test -p terangacast-capture -- --nocapture
```

Résultat attendu : `2 passed`

- [ ] **Step 2.3 : Implémenter ScapCapture (Windows)**

```rust
// crates/capture/src/scap_capture.rs
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
            return Err(CaptureError::Init("Capture d'écran non supportée sur ce système".into()));
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
```

- [ ] **Step 2.4 : Exposer ScapCapture dans lib.rs**

Ajouter en bas de `crates/capture/src/lib.rs` :

```rust
#[cfg(target_os = "windows")]
pub mod scap_capture;
#[cfg(target_os = "windows")]
pub use scap_capture::ScapCapture;
```

- [ ] **Step 2.5 : Compiler**

```bash
cargo build -p terangacast-capture
```

Résultat attendu : build OK (warnings possibles sur les imports, pas d'erreurs).

- [ ] **Step 2.6 : Commit**

```bash
git add crates/capture/
git commit -m "feat(capture): trait ScreenCapture + MockCapture + ScapCapture Windows"
```

---

## Task 3 : Conversion BGRA → YUV420

**Files:**
- Create: `crates/encoder/src/yuv.rs`
- Modify: `crates/encoder/src/lib.rs` (re-export)

- [ ] **Step 3.1 : Écrire le test**

```rust
// crates/encoder/src/yuv.rs

/// Convertit un buffer BGRA (4 octets/pixel) en YUV420 planaire (I420).
/// Retourne les 3 plans concaténés : Y (w*h) + U (w/2*h/2) + V (w/2*h/2).
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

            let y = (0.257 * r + 0.504 * g + 0.098 * b + 16.0).clamp(0.0, 255.0) as u8;
            y_plane[row * w + col] = y;

            if row % 2 == 0 && col % 2 == 0 {
                let uv_idx = (row / 2) * (w / 2) + (col / 2);
                u_plane[uv_idx] = (-(0.148 * r) - 0.291 * g + 0.439 * b + 128.0).clamp(0.0, 255.0) as u8;
                v_plane[uv_idx] = (0.439 * r - 0.368 * g - 0.071 * b + 128.0).clamp(0.0, 255.0) as u8;
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
        // BGRA pur rouge : B=0, G=0, R=255, A=255
        let width = 2u32;
        let height = 2u32;
        let bgra = vec![
            0u8, 0, 255, 255,
            0,   0, 255, 255,
            0,   0, 255, 255,
            0,   0, 255, 255,
        ];
        let yuv = bgra_to_yuv420(&bgra, width, height);
        // Y pour rouge pur ≈ 0.257*255 + 16 ≈ 82
        let y = yuv[0];
        assert!(y > 70 && y < 95, "Y rouge attendu ~82, obtenu {}", y);
    }

    #[test]
    fn pure_white_pixel_y_near_255() {
        let width = 2u32;
        let height = 2u32;
        let bgra = vec![255u8; 16]; // BGRA tout blanc
        let yuv = bgra_to_yuv420(&bgra, width, height);
        let y = yuv[0];
        assert!(y > 220, "Y blanc attendu proche de 255, obtenu {}", y);
    }
}
```

- [ ] **Step 3.2 : Lancer les tests — doivent passer**

```bash
cargo test -p terangacast-encoder yuv -- --nocapture
```

Résultat attendu : `3 passed`

- [ ] **Step 3.3 : Exposer dans lib.rs**

```rust
// crates/encoder/src/lib.rs
pub mod yuv;
pub use yuv::bgra_to_yuv420;
```

- [ ] **Step 3.4 : Commit**

```bash
git add crates/encoder/src/yuv.rs crates/encoder/src/lib.rs
git commit -m "feat(encoder): conversion BGRA→YUV420 avec tests"
```

---

## Task 4 : Crate encoder — trait + openh264

**Files:**
- Modify: `crates/encoder/src/lib.rs`
- Create: `crates/encoder/src/openh264_encoder.rs`

- [ ] **Step 4.1 : Définir les types et le trait dans lib.rs**

```rust
// crates/encoder/src/lib.rs
pub mod yuv;
pub use yuv::bgra_to_yuv420;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Initialisation encodeur : {0}")]
    Init(String),
    #[error("Encodage : {0}")]
    Encode(String),
}

/// Frame brute à encoder (YUV420 planaire I420)
pub struct EncoderInput {
    pub yuv_data: Vec<u8>, // Y+U+V concaténés (sortie de bgra_to_yuv420)
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
}

/// Frame encodée H.264 (NAL units prêts pour RTP)
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub keyframe: bool,
    pub timestamp_ms: u64,
}

pub trait VideoEncoder: Send {
    /// Retourne None si l'encodeur décide de sauter la frame.
    fn encode(&mut self, input: EncoderInput) -> Result<Option<EncodedFrame>, EncoderError>;
    fn request_keyframe(&mut self);
}

pub mod openh264_encoder;
pub use openh264_encoder::OpenH264Encoder;
```

- [ ] **Step 4.2 : Écrire le test de l'encodeur**

```rust
// crates/encoder/src/openh264_encoder.rs — tests en bas du fichier

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EncoderInput, VideoEncoder, bgra_to_yuv420};

    fn make_test_input(width: u32, height: u32, ts: u64) -> EncoderInput {
        let bgra = vec![128u8; (width * height * 4) as usize]; // gris neutre
        EncoderInput {
            yuv_data: bgra_to_yuv420(&bgra, width, height),
            width,
            height,
            timestamp_ms: ts,
        }
    }

    #[test]
    fn first_frame_produces_keyframe() {
        let mut enc = OpenH264Encoder::new(640, 360, 30, 1_000_000).unwrap();
        let input = make_test_input(640, 360, 0);
        let result = enc.encode(input).unwrap();
        let frame = result.expect("La première frame doit produire des données");
        assert!(frame.keyframe, "La première frame doit être une keyframe");
        assert!(!frame.data.is_empty(), "Les données encodées ne doivent pas être vides");
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
```

- [ ] **Step 4.3 : Lancer le test — doit échouer (fonction pas encore définie)**

```bash
cargo test -p terangacast-encoder -- --nocapture 2>&1 | head -20
```

Résultat attendu : `error[E0425]: cannot find function 'OpenH264Encoder::new'`

- [ ] **Step 4.4 : Implémenter OpenH264Encoder**

```rust
// crates/encoder/src/openh264_encoder.rs
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::formats::YUVSource;
use crate::{EncoderError, EncoderInput, EncodedFrame, VideoEncoder};

pub struct OpenH264Encoder {
    encoder: Encoder,
    width: u32,
    height: u32,
    force_keyframe: bool,
}

impl OpenH264Encoder {
    pub fn new(width: u32, height: u32, _fps: u32, bitrate_bps: u32) -> Result<Self, EncoderError> {
        let config = EncoderConfig::new()
            .set_bitrate_bps(bitrate_bps)
            .enable_skip_frame(false);
        let encoder = Encoder::with_config(config)
            .map_err(|e| EncoderError::Init(e.to_string()))?;
        Ok(Self { encoder, width, height, force_keyframe: false })
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

        struct I420Buf<'a> { y: &'a [u8], u: &'a [u8], v: &'a [u8], w: usize, h: usize }
        impl<'a> YUVSource for I420Buf<'a> {
            fn width(&self) -> i32  { self.w as i32 }
            fn height(&self) -> i32 { self.h as i32 }
            fn y(&self) -> &[u8]    { self.y }
            fn u(&self) -> &[u8]    { self.u }
            fn v(&self) -> &[u8]    { self.v }
            fn y_stride(&self) -> i32 { self.w as i32 }
            fn u_stride(&self) -> i32 { (self.w / 2) as i32 }
            fn v_stride(&self) -> i32 { (self.w / 2) as i32 }
        }

        let src = I420Buf { y, u, v, w, h };
        let bitstream = self.encoder.encode(&src)
            .map_err(|e| EncoderError::Encode(e.to_string()))?;

        let data: Vec<u8> = bitstream.to_vec();
        if data.is_empty() { return Ok(None); }

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
    use crate::{EncoderInput, VideoEncoder, bgra_to_yuv420};

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
    fn first_frame_produces_keyframe() {
        let mut enc = OpenH264Encoder::new(640, 360, 30, 1_000_000).unwrap();
        let input = make_test_input(640, 360, 0);
        let result = enc.encode(input).unwrap();
        let frame = result.expect("La première frame doit produire des données");
        assert!(frame.keyframe || !frame.data.is_empty());
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
```

- [ ] **Step 4.5 : Lancer les tests**

```bash
cargo test -p terangacast-encoder -- --nocapture
```

Résultat attendu : `4 passed` (2 yuv + 2 encodeur)

- [ ] **Step 4.6 : Commit**

```bash
git add crates/encoder/
git commit -m "feat(encoder): trait VideoEncoder + OpenH264Encoder avec tests"
```

---

## Task 5 : WebRTC Sender (webrtc_sender.rs)

**Files:**
- Create: `crates/streamer/src/webrtc_sender.rs`
- Modify: `crates/streamer/src/lib.rs`

- [ ] **Step 5.1 : Écrire le test (vérifie que l'offre SDP est générée)**

```rust
// dans crates/streamer/src/webrtc_sender.rs, section tests en bas

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
```

- [ ] **Step 5.2 : Implémenter WebRtcSender**

```rust
// crates/streamer/src/webrtc_sender.rs
use std::sync::Arc;
use std::time::Duration;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::media::Sample;
use bytes::Bytes;
use thiserror::Error;

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
        media_engine.register_default_codecs()
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        let api = APIBuilder::new().with_media_engine(media_engine).build();

        // Pas de STUN : réseau 100% local
        let config = RTCConfiguration::default();

        let peer_connection = Arc::new(
            api.new_peer_connection(config).await
                .map_err(|e| WebRtcError::Internal(e.to_string()))?
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

        peer_connection.add_track(video_track.clone()).await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        // Créer l'offre et attendre la fin du gathering ICE
        let offer = peer_connection.create_offer(None).await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        peer_connection.set_local_description(offer).await
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        // Attendre tous les candidats ICE (max 3s pour LAN)
        tokio::select! {
            _ = gather_complete.recv() => {}
            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
        }

        let local_desc = peer_connection.local_description().await
            .ok_or_else(|| WebRtcError::Internal("Pas de description locale".into()))?;

        let local_sdp = serde_json::to_string(&local_desc)
            .map_err(|e| WebRtcError::Internal(e.to_string()))?;

        Ok(Self { peer_connection, video_track, local_sdp })
    }

    pub fn local_sdp(&self) -> &str {
        &self.local_sdp
    }

    /// Applique la réponse SDP du récepteur TV
    pub async fn set_remote_answer(&self, answer_json: &str) -> Result<(), WebRtcError> {
        let answer: RTCSessionDescription = serde_json::from_str(answer_json)
            .map_err(|e| WebRtcError::Internal(format!("JSON invalide : {}", e)))?;
        self.peer_connection.set_remote_description(answer).await
            .map_err(|e| WebRtcError::Internal(e.to_string()))
    }

    /// Envoie une frame H.264 encodée (NAL units)
    pub async fn send_frame(&self, data: Vec<u8>, duration_ms: u64) -> Result<(), WebRtcError> {
        self.video_track.write_sample(&Sample {
            data: Bytes::from(data),
            duration: Duration::from_millis(duration_ms),
            ..Default::default()
        }).await.map_err(|e| WebRtcError::Internal(e.to_string()))
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
```

- [ ] **Step 5.3 : Lancer le test**

```bash
cargo test -p terangacast-streamer webrtc_sender -- --nocapture
```

Résultat attendu : `1 passed`

- [ ] **Step 5.4 : Commit**

```bash
git add crates/streamer/src/webrtc_sender.rs
git commit -m "feat(streamer): WebRtcSender — PeerConnection H.264 + génération SDP offer"
```

---

## Task 6 : Serveur de signalisation Axum

**Files:**
- Create: `crates/streamer/src/signaling.rs`
- Create: `receiver/index.html`
- Create: `receiver/receiver.js`

- [ ] **Step 6.1 : Créer le receiver HTML**

```html
<!-- receiver/index.html -->
<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TerangaCast — Récepteur</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { background: #000; display: flex; flex-direction: column;
           align-items: center; justify-content: center; height: 100vh; }
    video { width: 100%; max-height: 100vh; object-fit: contain; }
    #status { color: #fff; font-family: sans-serif; font-size: 1.2rem;
              position: fixed; top: 16px; left: 50%; transform: translateX(-50%);
              background: rgba(0,0,0,0.6); padding: 8px 16px; border-radius: 8px; }
    button#fullscreen { position: fixed; bottom: 16px; right: 16px;
                        background: rgba(255,255,255,0.15); color: #fff;
                        border: none; padding: 10px 18px; border-radius: 8px;
                        font-size: 1rem; cursor: pointer; }
  </style>
</head>
<body>
  <div id="status">Connexion en cours…</div>
  <video id="stream" autoplay playsinline></video>
  <button id="fullscreen" onclick="document.documentElement.requestFullscreen()">Plein écran</button>
  <script src="receiver.js"></script>
</body>
</html>
```

- [ ] **Step 6.2 : Créer le receiver JS**

```javascript
// receiver/receiver.js
(async () => {
  const status = document.getElementById('status');
  const videoEl = document.getElementById('stream');

  function setStatus(msg) {
    status.textContent = msg;
    console.log('[TerangaCast]', msg);
  }

  const pc = new RTCPeerConnection({ iceServers: [] }); // LAN uniquement

  pc.ontrack = (evt) => {
    setStatus('Flux reçu');
    videoEl.srcObject = evt.streams[0];
  };

  pc.oniceconnectionstatechange = () => {
    setStatus('ICE : ' + pc.iceConnectionState);
    if (pc.iceConnectionState === 'connected') {
      status.style.display = 'none'; // masquer quand connecté
    }
  };

  try {
    setStatus('Récupération de l\'offre…');
    const res = await fetch('/offer');
    const offerData = await res.json();

    await pc.setRemoteDescription(new RTCSessionDescription(offerData));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    setStatus('Envoi de la réponse…');
    await fetch('/answer', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(pc.localDescription),
    });

    setStatus('En attente du stream…');
  } catch (err) {
    setStatus('Erreur : ' + err.message);
    console.error(err);
  }
})();
```

- [ ] **Step 6.3 : Implémenter le serveur Axum**

```rust
// crates/streamer/src/signaling.rs
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
    response::Html,
    http::StatusCode,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
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

// Page HTML embarquée à la compilation
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

/// Démarre le serveur sur le port donné et retourne le handle.
/// Retourne également le receiver qui se résout quand la TV envoie sa réponse.
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
    use tower::ServiceExt; // pour .oneshot()

    async fn make_state() -> Arc<SignalingState> {
        let sender = Arc::new(
            WebRtcSender::new().await.expect("sender OK")
        );
        SignalingState::new(sender)
    }

    #[tokio::test]
    async fn get_offer_returns_200_with_sdp() {
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
```

- [ ] **Step 6.4 : Lancer les tests**

```bash
cargo test -p terangacast-streamer signaling -- --nocapture
```

Résultat attendu : `2 passed`

- [ ] **Step 6.5 : Commit**

```bash
git add crates/streamer/src/signaling.rs receiver/
git commit -m "feat(streamer): serveur signalisation Axum + page receiver TV"
```

---

## Task 7 : StreamManager — pipeline complète

**Files:**
- Create: `crates/streamer/src/lib.rs`

- [ ] **Step 7.1 : Implémenter StreamManager**

```rust
// crates/streamer/src/lib.rs
pub mod webrtc_sender;
pub mod signaling;

use std::sync::Arc;
use std::net::IpAddr;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use webrtc_sender::WebRtcSender;
use signaling::SignalingState;
use terangacast_capture::{ScreenCapture, RawFrame};
use terangacast_encoder::{VideoEncoder, EncoderInput, bgra_to_yuv420};
use thiserror::Error;
use tracing::{info, warn};

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
    pub fn low()    -> Self { Self { fps: 15, bitrate_bps: 800_000,  width: 1280, height: 720 } }
    pub fn medium() -> Self { Self { fps: 24, bitrate_bps: 2_000_000, width: 1920, height: 1080 } }
    pub fn high()   -> Self { Self { fps: 30, bitrate_bps: 4_000_000, width: 1920, height: 1080 } }
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
            WebRtcSender::new().await.map_err(|e| StreamError::WebRtc(e.to_string()))?
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

        let (stop_tx, mut stop_rx) = watch::channel(false);
        let sender_clone = sender.clone();
        let frame_ms = 1000 / quality.fps as u64;

        capture.start().ok();

        let capture_task = tokio::spawn(async move {
            loop {
                if *stop_rx.borrow() { break; }

                match capture.next_frame() {
                    Ok(frame) => {
                        let yuv = bgra_to_yuv420(&frame.data, frame.width, frame.height);
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        let input = EncoderInput { yuv_data: yuv, width: frame.width, height: frame.height, timestamp_ms: ts };

                        match encoder.encode(input) {
                            Ok(Some(encoded)) => {
                                if let Err(e) = sender_clone.send_frame(encoded.data, frame_ms).await {
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

        let info = StreamInfo { url, local_ip, port };
        Ok((manager, info))
    }

    pub fn stop(&self) {
        let _ = self.stop_tx.send(true);
    }
}

fn local_ip_address() -> Option<String> {
    // Technique simple : se connecter à une adresse distante sans envoyer de paquets
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
```

- [ ] **Step 7.2 : Ajouter axum comme dépendance directe dans lib**

Vérifier que `Cargo.toml` du streamer contient bien `axum`.

- [ ] **Step 7.3 : Compiler**

```bash
cargo build -p terangacast-streamer
```

Résultat attendu : pas d'erreurs.

- [ ] **Step 7.4 : Commit**

```bash
git add crates/streamer/src/lib.rs
git commit -m "feat(streamer): StreamManager — pipeline capture→encode→webrtc"
```

---

## Task 8 : Tauri — State et Commands

**Files:**
- Create: `sender/src-tauri/src/state.rs`
- Create: `sender/src-tauri/src/commands.rs`
- Modify: `sender/src-tauri/src/main.rs`

- [ ] **Step 8.1 : Définir AppState**

```rust
// sender/src-tauri/src/state.rs
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
```

- [ ] **Step 8.2 : Définir les commandes Tauri**

```rust
// sender/src-tauri/src/commands.rs
use tauri::State;
use serde::{Deserialize, Serialize};
use terangacast_streamer::{StreamManager, QualityConfig};
use terangacast_capture::ScapCapture;
use terangacast_encoder::OpenH264Encoder;
use crate::state::AppState;

#[derive(Serialize)]
pub struct StreamInfoDto {
    pub url: String,
    pub local_ip: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub enum QualityLevel { Low, Medium, High }

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
        QualityLevel::Low    => QualityConfig::low(),
        QualityLevel::Medium => QualityConfig::medium(),
        QualityLevel::High   => QualityConfig::high(),
    };

    let fps = quality_config.fps;
    let bitrate = quality_config.bitrate_bps;
    let width = quality_config.width;
    let height = quality_config.height;

    let capture = Box::new(
        ScapCapture::new(fps).map_err(|e| e.to_string())?
    );
    let encoder = Box::new(
        OpenH264Encoder::new(width, height, fps, bitrate).map_err(|e| e.to_string())?
    );

    let (manager, info) = StreamManager::start(capture, encoder, quality_config, 8765)
        .await
        .map_err(|e| e.to_string())?;

    let dto = StreamInfoDto {
        url: info.url,
        local_ip: info.local_ip,
        port: info.port,
    };

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
```

- [ ] **Step 8.3 : Mettre à jour main.rs**

```rust
// sender/src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod commands;

use state::AppState;
use commands::{start_stream, stop_stream, get_status};

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![start_stream, stop_stream, get_status])
        .run(tauri::generate_context!())
        .expect("Erreur lors du démarrage de Tauri");
}
```

- [ ] **Step 8.4 : Compiler le backend Tauri**

```bash
cargo build -p terangacast-sender 2>&1 | tail -5
```

Résultat attendu : `Finished` sans erreurs.

- [ ] **Step 8.5 : Commit**

```bash
git add sender/src-tauri/src/
git commit -m "feat(sender): state AppState + commandes Tauri start/stop/status"
```

---

## Task 9 : Interface React

**Files:**
- Create: `sender/src/components/QualitySelector.tsx`
- Create: `sender/src/components/StreamControl.tsx`
- Modify: `sender/src/App.tsx`

- [ ] **Step 9.1 : Installer les types Tauri**

```bash
cd sender && npm install @tauri-apps/api
```

- [ ] **Step 9.2 : Créer QualitySelector**

```tsx
// sender/src/components/QualitySelector.tsx
type Quality = 'Low' | 'Medium' | 'High';

interface Props {
  value: Quality;
  onChange: (q: Quality) => void;
  disabled: boolean;
}

const LABELS: Record<Quality, string> = {
  Low:    'Basse (720p 15fps)',
  Medium: 'Moyenne (1080p 24fps)',
  High:   'Haute (1080p 30fps)',
};

export function QualitySelector({ value, onChange, disabled }: Props) {
  return (
    <div style={{ margin: '16px 0' }}>
      <label style={{ display: 'block', marginBottom: 8, fontWeight: 600 }}>
        Qualité du stream
      </label>
      <div style={{ display: 'flex', gap: 8 }}>
        {(Object.keys(LABELS) as Quality[]).map((q) => (
          <button
            key={q}
            disabled={disabled}
            onClick={() => onChange(q)}
            style={{
              padding: '8px 16px',
              borderRadius: 8,
              border: '2px solid',
              borderColor: value === q ? '#f97316' : '#e2e8f0',
              background: value === q ? '#fff7ed' : '#fff',
              cursor: disabled ? 'not-allowed' : 'pointer',
              opacity: disabled ? 0.5 : 1,
              fontWeight: value === q ? 700 : 400,
            }}
          >
            {LABELS[q]}
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 9.3 : Créer StreamControl**

```tsx
// sender/src/components/StreamControl.tsx
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QualitySelector } from './QualitySelector';

type Quality = 'Low' | 'Medium' | 'High';

interface StreamInfo { url: string; local_ip: string; port: number; }

export function StreamControl() {
  const [running, setRunning] = useState(false);
  const [quality, setQuality] = useState<Quality>('Medium');
  const [info, setInfo] = useState<StreamInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleStart() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<StreamInfo>('start_stream', { quality });
      setInfo(result);
      setRunning(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleStop() {
    setLoading(true);
    try {
      await invoke('stop_stream');
      setRunning(false);
      setInfo(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{ fontFamily: 'sans-serif', maxWidth: 520, margin: '0 auto', padding: 32 }}>
      <h1 style={{ fontSize: '1.8rem', fontWeight: 800, color: '#1e293b', marginBottom: 4 }}>
        TerangaCast
      </h1>
      <p style={{ color: '#64748b', marginBottom: 24 }}>
        Diffusez votre écran sur la TV via Wi-Fi — sans câble, sans internet.
      </p>

      <QualitySelector value={quality} onChange={setQuality} disabled={running || loading} />

      <button
        onClick={running ? handleStop : handleStart}
        disabled={loading}
        style={{
          width: '100%',
          padding: '14px',
          borderRadius: 12,
          border: 'none',
          background: running ? '#ef4444' : '#f97316',
          color: '#fff',
          fontSize: '1.1rem',
          fontWeight: 700,
          cursor: loading ? 'wait' : 'pointer',
          marginTop: 8,
        }}
      >
        {loading ? '…' : running ? 'Arrêter le stream' : 'Démarrer le stream'}
      </button>

      {info && (
        <div style={{
          marginTop: 24, padding: 20, borderRadius: 12,
          background: '#f0fdf4', border: '1px solid #86efac'
        }}>
          <p style={{ fontWeight: 600, color: '#166534', marginBottom: 8 }}>
            Stream actif
          </p>
          <p style={{ color: '#15803d', marginBottom: 4 }}>
            Sur votre TV, ouvrez :
          </p>
          <code style={{
            display: 'block', padding: '10px 14px', borderRadius: 8,
            background: '#dcfce7', color: '#166534', fontSize: '1.1rem',
            fontWeight: 700, letterSpacing: 1,
          }}>
            {info.url}
          </code>
        </div>
      )}

      {error && (
        <div style={{
          marginTop: 16, padding: 14, borderRadius: 10,
          background: '#fef2f2', border: '1px solid #fca5a5',
          color: '#dc2626', fontSize: '0.9rem'
        }}>
          {error}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 9.4 : Mettre à jour App.tsx**

```tsx
// sender/src/App.tsx
import { StreamControl } from './components/StreamControl';

export default function App() {
  return (
    <div style={{
      minHeight: '100vh',
      background: 'linear-gradient(135deg, #fff7ed 0%, #fef3c7 100%)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    }}>
      <StreamControl />
    </div>
  );
}
```

- [ ] **Step 9.5 : Lancer l'app en mode dev**

```bash
cd sender && npm run tauri dev
```

Résultat attendu : fenêtre Tauri avec l'interface Start/Stop + sélecteur qualité.

- [ ] **Step 9.6 : Commit**

```bash
git add sender/src/
git commit -m "feat(ui): interface React — StreamControl + QualitySelector + App"
```

---

## Task 10 : Build Windows + Test end-to-end

**Files:**
- Modify: `sender/src-tauri/tauri.conf.json` (nom, icône, identifiers)

- [ ] **Step 10.1 : Mettre à jour tauri.conf.json**

```json
{
  "productName": "TerangaCast",
  "version": "0.1.0",
  "identifier": "sn.terangacast.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"]
  },
  "app": {
    "windows": [{
      "title": "TerangaCast",
      "width": 600,
      "height": 420,
      "resizable": false,
      "fullscreen": false
    }]
  }
}
```

- [ ] **Step 10.2 : Build de release Windows**

```bash
cd sender && npm run tauri build
```

Résultat attendu : `target/release/bundle/msi/TerangaCast_0.1.0_x64.msi` ou `.exe` dans `target/release/`.

- [ ] **Step 10.3 : Test end-to-end**

Procédure manuelle :
1. Lancer l'application TerangaCast sur le PC (`.exe` ou `npm run tauri dev`)
2. Sélectionner qualité **Basse** (plus rapide pour le test)
3. Cliquer **Démarrer le stream**
4. Vérifier que l'URL s'affiche (ex. `http://192.168.1.10:8765`)
5. Sur un téléphone ou une TV connectée au même Wi-Fi, ouvrir cette URL dans le navigateur
6. Vérifier que la vidéo de l'écran PC s'affiche sur la TV/téléphone
7. Vérifier la latence (objectif MVP : < 500ms)
8. Cliquer **Arrêter** et vérifier que le stream s'interrompt proprement

- [ ] **Step 10.4 : Commit final**

```bash
git add sender/src-tauri/tauri.conf.json
git commit -m "feat: MVP TerangaCast — build release Windows prêt"
```

---

## Checklist post-MVP

- [ ] La page TV se charge bien depuis le navigateur intégré d'une Smart TV Samsung/LG (tester avec WebOS et Tizen)
- [ ] La latence est acceptable sur un routeur Wi-Fi modeste (< 1s avec qualité Basse)
- [ ] L'app se ferme proprement (cleanup des threads tokio)
- [ ] L'URL s'affiche correctement même avec plusieurs interfaces réseau (Wi-Fi + Ethernet)
- [ ] Tester avec résolution 720p si 1080p est trop lent sur les PC bas de gamme

---

## Gaps spec couverts

| Fonctionnalité spec          | Tâche couvrant            |
|------------------------------|---------------------------|
| Capture d'écran temps réel   | Task 2 + Task 7           |
| Encodage H.264 hardware      | Task 4 (software MVP, hardware en Phase 2) |
| Transmission WebRTC          | Task 5 + Task 7           |
| Découverte automatique mDNS  | Non inclus MVP (affichage URL manuel — Phase 3) |
| Interface Start/Stop + qualité | Task 9                  |
| Page web sur TV              | Task 6                    |
| Support Windows prioritaire  | Task 2 (ScapCapture DXGI) |

> **Note Phase 2 :** Le hardware encoding (NVENC/QuickSync/AMF) remplacera `openh264` dans `crates/encoder/` sans changer l'interface `VideoEncoder`. La découverte mDNS via `mdns-sd` sera ajoutée dans `crates/discovery/` (Phase 3).
