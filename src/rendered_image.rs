use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use image::ImageEncoder;

use crate::{Error, MAX_PIXELS};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Owned RGB raster ready for PNG encoding.
pub struct RenderedImage {
    /// Raster width in pixels.
    pub width: u32,
    /// Raster height in pixels.
    pub height: u32,
    /// RGB bytes, top-to-bottom rows and left-to-right columns.
    pub pixels: Vec<u8>,
}

impl RenderedImage {
    /// Encode PNG to a sibling temporary file, then atomically replace the destination.
    /// Existing output is preserved if validation, encoding, or flushing fails.
    ///
    /// # Errors
    /// Returns `Error::Settings` if dimensions do not match the RGB buffer or pixel limit,
    /// `Error::Io` if temporary-file creation, flushing, or replacement fails, and
    /// `Error::Image` if PNG encoding fails.
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let count = (self.width as usize).checked_mul(self.height as usize);
        if self.width == 0
            || self.height == 0
            || count.is_none_or(|count| {
                count > MAX_PIXELS || count.checked_mul(3) != Some(self.pixels.len())
            })
        {
            return Err(Error::Settings(
                "image dimensions must match the RGB byte count and pixel budget".into(),
            ));
        }
        let file_name = path
            .file_name()
            .ok_or_else(|| Error::Settings("PNG output must name a file".into()))?;
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
                image::codecs::png::PngEncoder::new(&mut file)
                    .write_image(
                        &self.pixels,
                        self.width,
                        self.height,
                        image::ExtendedColorType::Rgb8,
                    )
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
            "temporary PNG name collisions; remove stale sibling .tmp files and retry",
        )))
    }
}
