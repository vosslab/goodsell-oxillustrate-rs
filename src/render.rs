//! Synchronous raster, shading, and outline composition.

use crate::shading::Shading;
use crate::{Error, RenderedImage, Scene, outlines, raster};

/// Render prepared geometry into owned, top-to-bottom RGB bytes.
///
/// # Errors
/// Returns `Error::EmptyScene` if no sphere covers the image and `Error::Allocation` if
/// raster or RGB storage cannot be reserved within the image limit.
pub fn render(scene: &Scene) -> Result<RenderedImage, Error> {
    let buffer = raster::rasterize(scene)?;
    let allocation_error = || Error::Allocation {
        width: scene.width,
        height: scene.height,
    };
    let byte_count = buffer
        .depth
        .len()
        .checked_mul(3)
        .ok_or_else(allocation_error)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(byte_count)
        .map_err(|_| allocation_error())?;
    let shading = Shading::new(scene, &buffer);
    for row in 0..scene.height as usize {
        for column in 0..scene.width as usize {
            let opacity = outlines::opacity(scene, &buffer, row, column);
            for channel in shading.color(row, column) {
                pixels.push(
                    ((1.0 - opacity) * channel * 255.0)
                        .trunc()
                        .clamp(0.0, 255.0) as u8,
                );
            }
        }
    }
    Ok(RenderedImage {
        width: scene.width,
        height: scene.height,
        pixels,
    })
}
