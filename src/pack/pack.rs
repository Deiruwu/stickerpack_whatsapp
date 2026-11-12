use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use anyhow::{bail, Context};
use serde_json::json;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};
use crate::consts::EMOJI;
use crate::converter::converter::Converter;
use crate::converter::loader::Loaded;
use crate::utils::slug;

pub struct Pack {
    pub id: String,
    name: String,
    author: String,
    stickers: Vec<(String, Vec<u8>)>,
    tray: Option<Vec<u8>>,
    animated: Option<bool>,
}

impl Pack {
    pub fn new(name: &str, author: &str) -> Self {
        Self {
            id: slug(name),
            name: name.to_string(),
            author: author.to_string(),
            stickers: Vec::new(),
            tray: None,
            animated: None,
        }
    }

    pub fn add(&mut self, path: &Path, conv: &Converter) -> anyhow::Result<()> {
        let loaded = Loaded::load(path).with_context(|| format!("leyendo {}", path.display()))?;
        let is_anim = matches!(loaded, Loaded::Anim(_));

        match self.animated {
            None => self.animated = Some(is_anim),
            Some(a) if a != is_anim => bail!(
                "WhatsApp no permite mezclar estáticos y animados en un pack ({} es {})",
                path.display(),
                if is_anim { "animado" } else { "estático" }
            ),
            _ => {}
        }

        let data = match loaded {
            Loaded::Still(img) => {
                if self.tray.is_none() {
                    self.tray = Some(conv.tray(&img)?);
                }
                conv.still(&img)
            }
            Loaded::Anim(frames) => {
                if self.tray.is_none() {
                    self.tray = Some(conv.tray(&frames[0].0)?);
                }
                conv.animated(frames)
            }
        }
            .with_context(|| format!("convirtiendo {}", path.display()))?;

        let name = format!("{:02}.webp", self.stickers.len() + 1);
        println!("Ok({} -> {name} ({} KB))", path.file_name().unwrap().display(), data.len() / 1024);
        self.stickers.push((name, data));
        Ok(())
    }

    fn contents(&self) -> anyhow::Result<Vec<u8>> {
        let contents = json!({
            "android_play_store_link": "",
            "ios_app_store_link": "",
            "sticker_packs": [{
                "identifier": self.id,
                "name": self.name,
                "publisher": self.author,
                "tray_image_file": "tray.png",
                "image_data_version": "1",
                "avoid_cache": false,
                "publisher_email": "",
                "publisher_website": "",
                "privacy_policy_website": "",
                "license_agreement_website": "",
                "animated_sticker_pack": self.animated.unwrap_or(false),
                "stickers": self.stickers.iter().map(|(n, _)| json!({
                    "image_file": n,
                    "emojis": [EMOJI],
                })).collect::<Vec<_>>(),
            }]
        });
        Ok(serde_json::to_vec_pretty(&contents)?)
    }

    /// Escribe los archivos sueltos y devuelve el zip.
    pub fn export(&self, out_dir: &Path) -> anyhow::Result<Vec<u8>> {
        let tray = self.tray.as_ref().context("pack sin stickers")?;
        let contents = self.contents()?;

        fs::create_dir_all(out_dir)?;
        for (n, d) in &self.stickers {
            fs::write(out_dir.join(n), d)?;
        }
        fs::write(out_dir.join("tray✔.png"), tray)?;
        fs::write(out_dir.join("contents.json"), &contents)?;

        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file("contents.json", opts)?;
        zip.write_all(&contents)?;
        zip.start_file("tray.png", opts)?;
        zip.write_all(tray)?;
        for (n, d) in &self.stickers {
            zip.start_file(n.as_str(), opts)?;
            zip.write_all(d)?;
        }
        let zip_bytes = zip.finish()?.into_inner();

        let zip_path = out_dir.join("pack.zip");
        fs::write(&zip_path, &zip_bytes)?;
        println!("\n{} ({} KB)", zip_path.display(), zip_bytes.len() / 1024);
        Ok(zip_bytes)
    }
}
