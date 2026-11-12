//! stickerpack: imágenes/GIFs -> pack de stickers WebP 512x512 -> zip -> QR en terminal.
//! Uso:
//! stickerpack fotos/ gif1.gif img.png --name "Mi pack" --author "Dei" --base height

pub mod consts;
pub mod model;
pub mod utils;
pub mod server;
pub mod converter;
pub mod pack;

use anyhow::{Result, bail};
use clap::Parser;

use crate::consts::*;
use crate::converter::converter::Converter;
use crate::model::cli::Cli;
use crate::pack::pack::Pack;
use crate::server::pack_server::PackServer;
use crate::utils::collect_inputs;


fn main() -> Result<()> {
    let cli = Cli::parse();

    let files = collect_inputs(&cli.inputs)?;
    if files.len() < MIN_STICKERS {
        bail!("WhatsApp exige al menos {MIN_STICKERS} stickers (hay {})", files.len());
    }
    if files.len() > MAX_STICKERS {
        bail!("WhatsApp admite como máximo {MAX_STICKERS} stickers (hay {})", files.len());
    }

    let conv = Converter::new(cli.base);
    let mut pack = Pack::new(&cli.name, &cli.author);
    for path in &files {
        pack.add(path, &conv)?;
    }

    let out_dir = std::env::temp_dir().join("stickerpack").join(&pack.id);
    let zip = pack.export(&out_dir)?;

    PackServer::new(zip, PORT).run()
}