#[derive(Clone, Debug, Eq, PartialEq)]
/// Owned RGB raster ready for PNG encoding.
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    /// RGB bytes, top-to-bottom rows and left-to-right columns.
    pub pixels: Vec<u8>,
}
