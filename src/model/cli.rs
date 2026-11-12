use std::path::PathBuf;
use clap::{Parser};
use crate::model::base_mode::BaseMode;

#[derive(Parser, Debug)]
#[command(about)]
pub struct Cli {
    /// Imágenes, GIFs o carpetas (png, jpg, gif, webp, bmp)
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Nombre del pack
    #[arg(short, long, default_value = "Mi pack")]
    pub name: String,

    /// Autor / publisher del pack
    #[arg(short, long, default_value = "Anónimo")]
    pub author: String,

    /// Dimensión base para escalar a 512 (la otra se centra en el lienzo)
    #[arg(short, long, value_enum, default_value_t = BaseMode::Height)]
    pub base: BaseMode,
}