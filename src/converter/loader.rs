use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use anyhow::bail;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, RgbaImage};

pub enum Loaded {
    /// Imagen estática
    Still(RgbaImage),
    /// Frames ya compuestos con su duración en ms
    Anim(Vec<(RgbaImage, u32)>),
}

impl Loaded {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let is_gif = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("gif"))
            .unwrap_or(false);

        if !is_gif {
            return Ok(Self::Still(image::open(path)?.to_rgba8()));
        }

        let dec = GifDecoder::new(BufReader::new(File::open(path)?))?;
        let frames = dec.into_frames().collect_frames()?;
        if frames.len() > 1 {
            let out = frames
                .into_iter()
                .map(|f| {
                    let (n, d) = f.delay().numer_denom_ms();
                    let ms = if d == 0 { 100 } else { n / d };
                    let ms = if ms <= 10 { 100 } else { ms };
                    (f.into_buffer(), ms)
                })
                .collect();
            return Ok(Self::Anim(out));
        }
        match frames.into_iter().next() {
            Some(f) => Ok(Self::Still(f.into_buffer())),
            None => bail!("GIF sin frames"),
        }
    }
}
