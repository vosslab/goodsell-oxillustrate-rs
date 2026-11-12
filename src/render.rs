//! Synchronous raster, shading, and outline composition.

use crate::shading::Shading;
use crate::{Error, RenderedImage, Scene, outlines, raster};

/// Render geometry into content-trimmed RGB bytes and optional straight alpha.
/// Trimming retains every atom/outline pixel plus three pixels of available canvas padding.
///
/// # Errors
/// Returns `Error::EmptyScene` if no sphere covers the image and `Error::Allocation` if
/// raster or RGB storage cannot be reserved within the image limit.
pub fn render(scene: &Scene) -> Result<RenderedImage, Error> {
    render_with_bounds(scene).map(|(image, _)| image)
}

/// Render the same trimmed image as [`render`], with its `[left, top]` canvas origin.
/// Paste the image at that pixel offset, without scaling, to restore the scene canvas.
///
/// # Errors
/// Returns the same raster and allocation errors as [`render`].
pub fn render_with_bounds(scene: &Scene) -> Result<(RenderedImage, [u32; 2]), Error> {
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
    let mut alpha = if scene.background.is_none() {
        let mut mask = Vec::new();
        mask.try_reserve_exact(buffer.depth.len())
            .map_err(|_| allocation_error())?;
        Some(mask)
    } else {
        None
    };
    let shading = Shading::new(scene, &buffer);
    // Bounds use coverage, so atoms matching an opaque background are still retained.
    let mut bounds = [scene.height as usize, scene.width as usize, 0, 0];
    for row in 0..scene.height as usize {
        for column in 0..scene.width as usize {
            let opacity = outlines::opacity(scene, &buffer, row, column);
            let occupied = buffer.atom_index[row * scene.width as usize + column].is_some();
            if occupied || opacity > 0.0 {
                bounds[0] = bounds[0].min(row);
                bounds[1] = bounds[1].min(column);
                bounds[2] = bounds[2].max(row + 1);
                bounds[3] = bounds[3].max(column + 1);
            }
            if let Some(mask) = &mut alpha {
                // Ceil preserves the existing truncated RGB compositor over white.
                mask.push(if occupied {
                    255
                } else {
                    (opacity * 255.0).ceil() as u8
                });
            }
            for channel in shading.color(row, column) {
                pixels.push(
                    ((1.0 - opacity) * channel * 255.0)
                        .trunc()
                        .clamp(0.0, 255.0) as u8,
                );
            }
        }
    }
    bounds[0] = bounds[0].saturating_sub(3);
    bounds[1] = bounds[1].saturating_sub(3);
    bounds[2] = (bounds[2] + 3).min(scene.height as usize);
    bounds[3] = (bounds[3] + 3).min(scene.width as usize);
    let width = (bounds[3] - bounds[1]) as u32;
    let height = (bounds[2] - bounds[0]) as u32;
    if width != scene.width || height != scene.height {
        let crop = |bytes: &[u8], channels: usize| -> Result<Vec<u8>, Error> {
            let row_bytes = width as usize * channels;
            let mut cropped = Vec::new();
            cropped
                .try_reserve_exact(row_bytes * height as usize)
                .map_err(|_| allocation_error())?;
            for row in bounds[0]..bounds[2] {
                let start = (row * scene.width as usize + bounds[1]) * channels;
                cropped.extend_from_slice(&bytes[start..start + row_bytes]);
            }
            Ok(cropped)
        };
        pixels = crop(&pixels, 3)?;
        alpha = alpha.as_deref().map(|bytes| crop(bytes, 1)).transpose()?;
    }
    let image = RenderedImage {
        width,
        height,
        dpi: scene.dpi,
        pixels,
        alpha,
    };
    Ok((image, [bounds[1] as u32, bounds[0] as u32]))
}
