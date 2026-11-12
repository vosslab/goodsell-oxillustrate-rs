use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use image::{ExtendedColorType, ImageEncoder};

use crate::{Error, MAX_PIXELS};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Owned RGB raster with optional straight alpha, ready for PNG or WebP encoding.
pub struct RenderedImage {
    /// Raster width in pixels.
    pub width: u32,
    /// Raster height in pixels.
    pub height: u32,
    /// RGB bytes, top-to-bottom rows and left-to-right columns.
    pub pixels: Vec<u8>,
    /// One unassociated alpha byte per pixel; None means fully opaque RGB.
    pub alpha: Option<Vec<u8>>,
}

enum Format {
    Png,
    WebP,
}

impl RenderedImage {
    /// Save as PNG or lossless WebP according to the case-insensitive filename extension.
    ///
    /// # Errors
    /// Returns `Error::Settings` for an unsupported extension, or the errors from `save_png`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        match path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => self.save_png(path),
            Some("webp") => self.save_webp(path),
            _ => Err(Error::Settings(
                "output filename must end in .png or .webp".into(),
            )),
        }
    }

    /// Encode lossless WebP, preserving alpha and atomically replacing the destination.
    ///
    /// # Errors
    /// Returns the same validation, I/O, and encoding error categories as `save_png`.
    pub fn save_webp(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.save_format(path.as_ref(), Format::WebP)
    }

    /// Encode PNG to a sibling temporary file, then atomically replace the destination.
    /// Existing output is preserved if validation, encoding, or flushing fails.
    ///
    /// # Errors
    /// Returns `Error::Settings` if dimensions do not match RGB/alpha buffers or pixel limit,
    /// `Error::Io` if temporary-file creation, flushing, or replacement fails, and
    /// `Error::Image` if encoding fails or `Error::Allocation` if RGBA packing cannot allocate.
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.save_format(path.as_ref(), Format::Png)
    }

    fn save_format(&self, path: &Path, format: Format) -> Result<(), Error> {
        let count = (self.width as usize).checked_mul(self.height as usize);
        if self.width == 0
            || self.height == 0
            || count.is_none_or(|count| {
                count > MAX_PIXELS
                    || count.checked_mul(3) != Some(self.pixels.len())
                    || self
                        .alpha
                        .as_ref()
                        .is_some_and(|alpha| alpha.len() != count)
            })
        {
            return Err(Error::Settings(
                "image dimensions must match the RGB/alpha byte counts and pixel budget".into(),
            ));
        }
        let file_name = path
            .file_name()
            .ok_or_else(|| Error::Settings("image output must name a file".into()))?;
        let mut rgba = Vec::new();
        let (bytes, color) = if let Some(alpha) = &self.alpha {
            let count = alpha.len().checked_mul(4).ok_or(Error::Allocation {
                width: self.width,
                height: self.height,
            })?;
            rgba.try_reserve_exact(count)
                .map_err(|_| Error::Allocation {
                    width: self.width,
                    height: self.height,
                })?;
            for (rgb, alpha) in self.pixels.as_chunks::<3>().0.iter().zip(alpha) {
                rgba.extend_from_slice(rgb);
                rgba.push(*alpha);
            }
            (rgba.as_slice(), ExtendedColorType::Rgba8)
        } else {
            (self.pixels.as_slice(), ExtendedColorType::Rgb8)
        };
        static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
        let io_error = |source| Error::Io {
            path: path.to_owned(),
            source,
        };
        for _ in 0..16 {
            let mut temp_name = file_name.to_os_string();
            temp_name.push(format!(
                ".{}.{}.tmp",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            let temporary = path.with_file_name(temp_name);
            let mut file = match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error(error)),
            };
            let result = (|| {
                match format {
                    Format::Png => image::codecs::png::PngEncoder::new(&mut file).write_image(
                        bytes,
                        self.width,
                        self.height,
                        color,
                    ),
                    Format::WebP => image::codecs::webp::WebPEncoder::new_lossless(&mut file)
                        .write_image(bytes, self.width, self.height, color),
                }
                .map_err(|error| Error::Image(Box::new(error)))?;
                file.flush().map_err(io_error)?;
                drop(file);
                fs::rename(&temporary, path).map_err(io_error)
            })();
            if result.is_err() {
                let _ = fs::remove_file(&temporary);
            }
            return result;
        }
        Err(io_error(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "temporary image name collisions; remove stale sibling .tmp files and retry",
        )))
    }
}
