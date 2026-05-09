// Currently unused since v0.10.0 switched to the ML pipeline in
// `cutout_ml.rs`. Kept around as a fast-path fallback for cases where
// the embedded U2Netp model is overkill (true uniform backgrounds).
#![allow(dead_code)]

//! Background removal for clipboard / file images ("Freistellen").
//!
//! Strategy: chroma-key the four corner regions. We assume the corners
//! are background — a reasonable bet for product shots, "flugzeug am
//! himmel"-style photos, screenshots with solid borders, and most logos.
//! The four corner medians are averaged into a single background color;
//! every pixel within `inner_dist` of that color becomes fully
//! transparent, every pixel beyond `outer_dist` stays opaque, and the
//! band between them is alpha-feathered to avoid hard cutout edges.
//!
//! Input formats: anything the `image` crate decodes — PNG, JPEG, WebP,
//! GIF, BMP. Output is always PNG (so the alpha channel survives even
//! if the input was a flat JPEG).
//!
//! Limits of this approach (call them out so the UI can manage
//! expectations): clutter / busy backgrounds, subjects coloured like
//! the background, fine details (hair, fur, foliage) — all those will
//! produce visibly bad cutouts. For pro-grade results the user wants ML
//! (rembg / U2Net), which is out of scope for a clipboard utility.

use anyhow::{Context, Result};
use image::{ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;

/// Same cap as the recolor module — keep work bounded on the UI thread.
const MAX_PIXELS: u32 = 16_000_000;

/// Square Euclidean distance in RGB. Below this, pixels are treated as
/// pure background → alpha 0. ~30 perceptual units feels right for
/// uniform skies / studio backdrops.
const DEFAULT_INNER_DIST_SQ: u32 = 30 * 30;

/// Square Euclidean distance in RGB. Above this, pixels are treated as
/// pure foreground → alpha kept (or set opaque). The 20-unit band
/// between inner and outer becomes a feathered edge.
const DEFAULT_OUTER_DIST_SQ: u32 = 50 * 50;

/// Side length of the corner sample square. 8×8 = 64 pixels per corner,
/// 256 total — enough for a stable median, small enough to fit even
/// 16×16 favicon-sized inputs.
const CORNER_SIDE: u32 = 8;

#[derive(Debug, Clone)]
pub struct CutoutResult {
    /// The detected background colour (median over corner samples).
    /// Currently unread by the IPC layer — kept around so future UI
    /// work ("we removed THIS colour, was that the right one?") doesn't
    /// need to re-decode the image.
    #[allow(dead_code)]
    pub background: (u8, u8, u8),
    /// Output PNG bytes, RGBA, alpha-feathered around the background.
    pub png: Vec<u8>,
}

/// Decode `image_bytes` (any format the `image` crate supports — PNG,
/// JPEG, WebP, GIF, BMP), knock out the corner-sampled background
/// colour, re-encode as RGBA PNG. Pure function — does no IO.
pub fn cut_out_background(image_bytes: &[u8]) -> Result<CutoutResult> {
    // `load_from_memory` sniffs the format from the magic bytes — we
    // don't need to know the source extension, which matters for the
    // file-cutout path where users pass JPEGs without extension hints.
    let img = image::load_from_memory(image_bytes)
        .context("decode image (unsupported format or corrupt)")?;
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        anyhow::bail!("empty image");
    }
    if w.saturating_mul(h) > MAX_PIXELS {
        anyhow::bail!("image too large to process ({}×{}), max 16 MP", w, h);
    }

    let rgba = img.to_rgba8();
    let bg = sample_corner_background(&rgba);
    let (br, bg_, bb) = (bg.0 as i32, bg.1 as i32, bg.2 as i32);

    let mut out: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    for (px_in, px_out) in rgba.pixels().zip(out.pixels_mut()) {
        let [r, g, b, a] = px_in.0;
        let dr = r as i32 - br;
        let dg = g as i32 - bg_;
        let db = b as i32 - bb;
        let dist_sq = (dr * dr + dg * dg + db * db) as u32;

        let new_alpha = if dist_sq <= DEFAULT_INNER_DIST_SQ {
            0u8
        } else if dist_sq >= DEFAULT_OUTER_DIST_SQ {
            a
        } else {
            // Linear feather between inner and outer. `t` goes 0..1
            // across the band; we multiply the source alpha by it so
            // already-translucent pixels stay translucent.
            let span = DEFAULT_OUTER_DIST_SQ - DEFAULT_INNER_DIST_SQ;
            let t = (dist_sq - DEFAULT_INNER_DIST_SQ) as f32 / span as f32;
            ((a as f32) * t).round().clamp(0.0, 255.0) as u8
        };
        *px_out = Rgba([r, g, b, new_alpha]);
    }

    let mut buf: Vec<u8> = Vec::with_capacity(image_bytes.len());
    out.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .context("encode PNG")?;
    Ok(CutoutResult { background: bg, png: buf })
}

/// Take the four corners (CORNER_SIDE × CORNER_SIDE patches), compute
/// the median R / G / B per channel across all gathered pixels, and
/// return that as the assumed background colour.
///
/// Median (not mean) is the right summary here because corners often
/// contain a few subject pixels (a wing tip, an antenna) — mean would
/// drag the estimate towards the subject, median ignores them.
fn sample_corner_background(rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> (u8, u8, u8) {
    let w = rgba.width();
    let h = rgba.height();
    let side = CORNER_SIDE.min(w.min(h));
    let mut rs = Vec::with_capacity((side * side * 4) as usize);
    let mut gs = Vec::with_capacity((side * side * 4) as usize);
    let mut bs = Vec::with_capacity((side * side * 4) as usize);

    let corners: [(u32, u32); 4] = [
        (0, 0),
        (w - side, 0),
        (0, h - side),
        (w - side, h - side),
    ];
    for (x0, y0) in corners {
        for dy in 0..side {
            for dx in 0..side {
                let p = rgba.get_pixel(x0 + dx, y0 + dy).0;
                if p[3] < 16 {
                    continue; // skip already-transparent corner pixels
                }
                rs.push(p[0]);
                gs.push(p[1]);
                bs.push(p[2]);
            }
        }
    }
    if rs.is_empty() {
        return (0, 0, 0);
    }
    rs.sort_unstable();
    gs.sort_unstable();
    bs.sort_unstable();
    let mid = rs.len() / 2;
    (rs[mid], gs[mid], bs[mid])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_png(w: u32, h: u32, color: [u8; 4]) -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(w, h, |_, _| Rgba(color));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).unwrap();
        buf
    }

    /// Image: blue background with a red square in the middle. Cutout
    /// should preserve the red square and remove the blue background.
    fn red_on_blue_png(w: u32, h: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(w, h, |x, y| {
            let in_subject =
                x > w / 4 && x < w * 3 / 4 && y > h / 4 && y < h * 3 / 4;
            if in_subject {
                Rgba([220, 30, 30, 255])
            } else {
                Rgba([40, 80, 200, 255])
            }
        });
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).unwrap();
        buf
    }

    #[test]
    fn detects_corner_background_color() {
        let png = solid_png(32, 32, [40, 80, 200, 255]);
        let res = cut_out_background(&png).unwrap();
        assert_eq!(res.background, (40, 80, 200));
    }

    #[test]
    fn removes_background_keeps_subject() {
        let png = red_on_blue_png(64, 64);
        let res = cut_out_background(&png).unwrap();
        let img = image::load_from_memory(&res.png).unwrap().to_rgba8();
        // Centre = subject (red, opaque)
        let centre = img.get_pixel(32, 32);
        assert_eq!(centre.0[3], 255, "subject pixel should stay opaque");
        // Corner = background (blue, removed)
        let corner = img.get_pixel(2, 2);
        assert_eq!(corner.0[3], 0, "background corner should become transparent");
    }

    #[test]
    fn rejects_oversized_images() {
        let big = solid_png(5000, 5000, [0, 0, 0, 255]);
        assert!(cut_out_background(&big).is_err());
    }

    #[test]
    fn handles_solid_image_gracefully() {
        // No subject — every pixel matches the background. Output should
        // simply be fully transparent, not crash.
        let png = solid_png(16, 16, [128, 128, 128, 255]);
        let res = cut_out_background(&png).unwrap();
        let img = image::load_from_memory(&res.png).unwrap().to_rgba8();
        for p in img.pixels() {
            assert_eq!(p.0[3], 0);
        }
    }

    #[test]
    fn ignores_already_transparent_corners() {
        // PNG with alpha=0 corners shouldn't bias the background sampler
        // toward pure black.
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(32, 32, |x, y| {
            let in_subject = x > 8 && x < 24 && y > 8 && y < 24;
            if in_subject {
                Rgba([200, 50, 50, 255])
            } else {
                Rgba([0, 0, 0, 0])
            }
        });
        let mut png = Vec::new();
        img.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
        let res = cut_out_background(&png).unwrap();
        // No opaque corner pixels → fallback (0,0,0) — that's the
        // documented behaviour, and it makes the cutout idempotent.
        assert_eq!(res.background, (0, 0, 0));
    }
}
