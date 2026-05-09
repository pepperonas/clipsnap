//! ML-based subject cutout — the "real" Freistellen.
//!
//! Drives a U2Netp ONNX model (~4.5 MB, embedded via `include_bytes!`)
//! through ONNX Runtime (via the `ort` crate) for cross-platform
//! inference. Same architecture as Python's `rembg`, just without
//! Python.
//!
//! The pipeline:
//!   1. Decode any image format → RGB.
//!   2. Resize to 320×320 (U2Net's input shape).
//!   3. Normalize with ImageNet mean / std (the values U2Net was
//!      trained against — anything else degrades quality noticeably).
//!   4. Run inference → single-channel saliency mask in [0, 1].
//!   5. Resize the mask back to the original dimensions.
//!   6. Apply mask as alpha on the original RGB → encode PNG.
//!
//! Quality vs. chroma-key: this *actually segments the subject* rather
//! than picking pixels by colour distance. Photos that destroy
//! chroma-key (airplane in gradient sky, person against cluttered
//! background, …) work cleanly here. The trade-off is latency
//! (~1–4 s on CPU for a typical-size photo) and ~30 MB binary growth
//! from the bundled ONNX Runtime native library.

use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, ImageFormat, Luma, Rgba};
use ndarray::Array4;
use ort::{session::Session, value::Value};
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

const MODEL_BYTES: &[u8] = include_bytes!("../models/u2netp.onnx");
const INPUT_SIZE: usize = 320;

/// ImageNet normalization — U2Net was trained against these stats; any
/// other normalization degrades the mask noticeably.
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Same cap as the chroma-key implementation — keeps work bounded on
/// the UI thread. Anything bigger gets rejected before we waste
/// several seconds on inference plus a giant final encode.
const MAX_PIXELS: u32 = 16_000_000;

/// ort `Session` is `!Sync` (it owns mutable runtime state internally),
/// so we wrap in a Mutex. `OnceLock` ensures we pay the model-build
/// cost (~150 ms) exactly once across the process lifetime — every
/// subsequent cutout reuses the same session.
static SESSION: OnceLock<Result<Mutex<Session>, String>> = OnceLock::new();

fn session() -> Result<&'static Mutex<Session>> {
    let cached = SESSION.get_or_init(|| {
        Session::builder()
            .map_err(|e| format!("ort session builder: {e}"))?
            .commit_from_memory(MODEL_BYTES)
            .map_err(|e| format!("commit U2Netp ONNX: {e}"))
            .map(Mutex::new)
    });
    cached.as_ref().map_err(|e| anyhow::anyhow!("model init: {e}"))
}

/// Run ML subject cutout on `image_bytes`. Accepts any format the
/// `image` crate decodes (PNG, JPEG, WebP, GIF, BMP). Returns a PNG
/// with the model-inferred subject mask applied as alpha.
pub fn cut_out_subject(image_bytes: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(image_bytes)
        .context("decode image (unsupported format or corrupt)")?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        anyhow::bail!("empty image");
    }
    if w.saturating_mul(h) > MAX_PIXELS {
        anyhow::bail!("image too large to process ({}×{}), max 16 MP", w, h);
    }

    // ── 1. Preprocess: resize → normalize → CHW tensor ───────────────
    // U2Net ignores aspect ratio and just resamples to a square; that's
    // how the original training pipeline works, and matching the
    // training behaviour gives the best masks.
    let resized = img
        .resize_exact(
            INPUT_SIZE as u32,
            INPUT_SIZE as u32,
            image::imageops::FilterType::Triangle,
        )
        .to_rgb8();

    let mut input = Array4::<f32>::zeros((1, 3, INPUT_SIZE, INPUT_SIZE));
    for y in 0..INPUT_SIZE {
        for x in 0..INPUT_SIZE {
            let p = resized.get_pixel(x as u32, y as u32);
            for c in 0..3 {
                let v = p.0[c] as f32 / 255.0;
                input[[0, c, y, x]] = (v - IMAGENET_MEAN[c]) / IMAGENET_STD[c];
            }
        }
    }

    // ── 2. Inference ────────────────────────────────────────────────
    let session = session()?;
    let mut session = session.lock().map_err(|e| anyhow::anyhow!("session lock: {e}"))?;
    let input_value = Value::from_array(input).context("build input value")?;
    let outputs = session
        .run(ort::inputs!["input.1" => input_value])
        .context("U2Netp inference")?;

    // U2Net exports six side outputs + a final fused output; the fused
    // result (the model's "best guess" mask) is at index 0.
    let (shape, mask_data) = outputs[0]
        .try_extract_tensor::<f32>()
        .context("read mask tensor")?;
    let expected = [1i64, 1, INPUT_SIZE as i64, INPUT_SIZE as i64];
    if shape.as_ref() != expected.as_slice() {
        anyhow::bail!(
            "unexpected mask shape {:?} (expected {:?})",
            shape,
            expected
        );
    }

    // ── 3. Postprocess: mask → 8-bit grayscale → resize back ─────────
    // U2Net's raw output is roughly in [0, 1] but can spill outside
    // (raw sigmoid logits in practice). Clamp matches what rembg does;
    // min-max stretching would amplify mask noise.
    let mut mask_bytes = Vec::with_capacity(INPUT_SIZE * INPUT_SIZE);
    for &v in mask_data {
        mask_bytes.push((v.clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    let mask_small: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(INPUT_SIZE as u32, INPUT_SIZE as u32, mask_bytes)
            .context("build mask image buffer")?;
    let mask = image::imageops::resize(
        &mask_small,
        w,
        h,
        image::imageops::FilterType::Triangle,
    );

    // ── 4. Composite: original RGB + mask alpha → RGBA PNG ───────────
    let rgb = img.to_rgb8();
    let mut out: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = rgb.get_pixel(x, y);
            let a = mask.get_pixel(x, y).0[0];
            out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], a]));
        }
    }

    let mut buf: Vec<u8> = Vec::with_capacity(image_bytes.len());
    out.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .context("encode PNG")?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_png(w: u32, h: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(w, h, |_, _| Rgba([128, 128, 128, 255]));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).unwrap();
        buf
    }

    #[test]
    fn rejects_oversized_images() {
        let big = solid_png(5000, 5000);
        assert!(cut_out_subject(&big).is_err());
    }

    #[test]
    fn rejects_corrupt_input() {
        // Random bytes — should fail at decode, not panic during ML.
        let junk = vec![0u8, 1, 2, 3, 4, 5];
        assert!(cut_out_subject(&junk).is_err());
    }

    /// Smoke test: run the full pipeline on a small synthetic image
    /// and confirm we get a valid PNG out. Doesn't validate mask
    /// quality (a 32×32 grey square is meaningless to U2Net) — just
    /// that the wiring works end-to-end.
    #[test]
    fn pipeline_runs_on_synthetic_input() {
        let png = solid_png(64, 64);
        let out = cut_out_subject(&png).expect("pipeline should succeed");
        // Output must decode as a valid RGBA PNG with the input dims.
        let decoded = image::load_from_memory(&out).expect("output is a valid PNG");
        assert_eq!(decoded.dimensions(), (64, 64));
        assert_eq!(decoded.color(), image::ColorType::Rgba8);
    }
}
