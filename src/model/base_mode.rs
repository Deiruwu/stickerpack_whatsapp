use clap::ValueEnum;

/// Qué dimensión de la imagen original se toma como base de la conversión.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum BaseMode {
    /// La altura pasa a 512 y el ancho se ajusta proporcionalmente.
    Height,
    /// El ancho pasa a 512 y la altura se ajusta proporcionalmente.
    Width,
}