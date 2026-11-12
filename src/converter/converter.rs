use std::io::Cursor;
use anyhow::{anyhow, bail};
use image::{imageops, RgbaImage, Rgba};
use image::imageops::FilterType;
use crate::consts::{MAX_ANIMATED_BYTES, MAX_ANIM_MS, MAX_ENCODE_FRAMES, MAX_STATIC_BYTES, MAX_TRAY_BYTES, MIN_FRAME_MS, STICKER_SIZE, TRAY_SIZE};
use crate::model::base_mode::BaseMode;

pub struct Converter {
    base: BaseMode,
}

impl Converter {
    pub fn new(base: BaseMode) -> Self {
        Self { base }
    }

    /// Escala usando `base` como dimensión base y centra en un lienzo cuadrado transparente.
    pub fn fit(&self, img: &RgbaImage, size: u32) -> RgbaImage {
        let (w, h) = img.dimensions();
        let (nw, nh) = match self.base {
            BaseMode::Height => (((w as f64 * size as f64 / h as f64).round() as u32).max(1), size),
            BaseMode::Width => (size, ((h as f64 * size as f64 / w as f64).round() as u32).max(1)),
        };
        let resized = imageops::resize(img, nw, nh, FilterType::Lanczos3);
        let mut canvas = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
        let ox = (size as i64 - nw as i64) / 2;
        let oy = (size as i64 - nh as i64) / 2;
        imageops::replace(&mut canvas, &resized, ox, oy);
        canvas
    }

    pub fn still(&self, img: &RgbaImage) -> anyhow::Result<Vec<u8>> {
        let img = self.fit(img, STICKER_SIZE);
        let (w, h) = img.dimensions();
        let enc = webp::Encoder::from_rgba(img.as_raw(), w, h);
        let mut q = 90.0_f32;
        loop {
            let data = enc.encode(q);
            if data.len() <= MAX_STATIC_BYTES {
                return Ok(data.to_vec());
            }
            if q <= 10.0 {
                bail!("no baja de {} KB ni con calidad mínima", MAX_STATIC_BYTES / 1024);
            }
            q -= 10.0;
        }
    }

    pub fn animated(&self, frames: Vec<(RgbaImage, u32)>) -> anyhow::Result<Vec<u8>> {
        if frames.is_empty() {
            bail!("animación vacía");
        }

        // Ajuste de un frame en los delays
        let (imgs, mut delays): (Vec<_>, Vec<_>) = frames
            .into_iter()
            .map(|(f, d)| (self.fit(&f, STICKER_SIZE), d))
            .unzip();
        delays.rotate_right(1);
        let frames: Vec<(RgbaImage, u32)> = imgs.into_iter().zip(delays).collect();

        // Picos de pausa intencionales (ej. parpadeos largos)
        let mut sorted: Vec<u32> = frames.iter().map(|f| f.1).collect();
        sorted.sort_unstable();
        let median = sorted[sorted.len() / 2];
        let is_peak = |d: u32| d > 300 && d > median * 3;

        // Mínimo por frame y recorte a 10 s
        let mut total = 0;
        let mut kept: Vec<(RgbaImage, u32)> = Vec::new();
        for (img, d) in frames {
            let d = d.max(MIN_FRAME_MS);
            if total + d > MAX_ANIM_MS {
                break;
            }
            total += d;
            kept.push((img, d));
        }
        let mut frames = kept;
        if frames.is_empty() {
            bail!("animación vacía tras recortar a 10 s");
        }

        // Reducción si hay demasiados frames, respetando picos
        if frames.len() > MAX_ENCODE_FRAMES {
            frames = frames
                .into_iter()
                .enumerate()
                .filter(|(i, (_, d))| is_peak(*d) || i % 2 == 0)
                .map(|(_, f)| f)
                .collect();
        }

        loop {
            for q in [75.0_f32, 60.0, 45.0, 30.0, 15.0] {
                let data = Self::encode_anim_once(&frames, q)?;
                if data.len() <= MAX_ANIMATED_BYTES {
                    return Ok(data);
                }
            }
            if frames.len() < 4 {
                bail!("la animación no baja de {} KB", MAX_ANIMATED_BYTES / 1024);
            }
            frames = frames
                .chunks(2)
                .map(|c| (c[0].0.clone(), c.iter().map(|f| f.1).sum()))
                .collect();
        }
    }

    pub fn encode_anim_once(frames: &[(RgbaImage, u32)], quality: f32) -> anyhow::Result<Vec<u8>> {
        use webp_animation::{prelude::*, AnimParams};
        let (w, h) = frames[0].0.dimensions();
        let opts = EncoderOptions {
            anim_params: AnimParams { loop_count: 0 },
            encoding_config: Some(EncodingConfig {
                encoding_type: EncodingType::Lossy(LossyEncodingConfig::default()),
                quality,
                method: 4,
            }),
            ..Default::default()
        };
        let mut enc = Encoder::new_with_options((w, h), opts).map_err(|e| anyhow!("{e:?}"))?;
        let mut t: i32 = 0;
        for (img, d) in frames {
            enc.add_frame(img.as_raw(), t).map_err(|e| anyhow!("{e:?}"))?;
            t += *d as i32;
        }
        let data = enc.finalize(t).map_err(|e| anyhow!("{e:?}"))?;
        Ok(data.to_vec())
    }

    pub fn tray(&self, img: &RgbaImage) -> anyhow::Result<Vec<u8>> {
        let tray = self.fit(img, TRAY_SIZE);
        let mut buf = Cursor::new(Vec::new());
        tray.write_to(&mut buf, image::ImageFormat::Png)?;
        let bytes = buf.into_inner();
        if bytes.len() > MAX_TRAY_BYTES {
            bail!("el icono de bandeja supera {} KB", MAX_TRAY_BYTES / 1024);
        }
        Ok(bytes)
    }
}