use super::file::File;
use anyhow::{anyhow, Context, Result};
use arboard::{Clipboard, SetExtLinux};
use image::{DynamicImage, ImageBuffer};
use std::thread::JoinHandle;

pub struct Clippy {
  pub clipboard: Clipboard,
}

/// A pending clipboard write, see [`Clippy::set`].
pub type ClipboardWrite = JoinHandle<Result<(), arboard::Error>>;

impl Clippy {
  pub fn new() -> Result<Self> {
    Ok(Clippy {
      clipboard: Clipboard::new().context("Failed to access the clipboard")?,
    })
  }

  pub fn get(&mut self) -> Result<File> {
    let image = self.clipboard.get_image();
    let text = self.clipboard.get_text();
    match (image, text) {
      (Ok(image), _) => {
        // Get the raw image data. (Width, Height, pixel values)
        let img_buf = ImageBuffer::from_raw(
          image.width as u32,
          image.height as u32,
          image.bytes.into_owned(),
        )
        .context("The image in the clipboard is invalid")?;
        // Convert the raw image data into a DynamicImage.
        let img = DynamicImage::ImageRgba8(img_buf);
        // Create a buffer to hold the image data.
        let mut buf = Vec::new();
        // Create a writer to write the image data to the buffer.
        let mut writer = std::io::Cursor::new(&mut buf);
        // Write the image data to the buffer.
        img
          .write_to(&mut writer, image::ImageFormat::Png)
          .context("Failed to encode the clipboard image")?;
        // Create a new file from the buffer.
        Ok(File::new(buf, "Clipboard".to_owned()))
      }
      (_, Ok(text)) => Ok(File::new(text.into_bytes(), "Clipboard".to_owned())),
      _ => Err(anyhow!(
        "No image or text in clipboard. Or clipboard does not contain supported format."
      )),
    }
  }

  /// Start setting the clipboard. On Linux the clipboard contents are only available while we keep serving them,
  /// so this happens on a separate thread, use [`Clippy::wait`] to wait for it before exiting.
  pub fn set(data: String) -> ClipboardWrite {
    std::thread::spawn(move || {
      Clipboard::new()?.set().wait().text(data)?;
      Ok::<(), arboard::Error>(())
    })
  }

  /// Wait for a clipboard write started with [`Clippy::set`] to finish.
  pub fn wait(handle: ClipboardWrite) -> Result<()> {
    handle
      .join()
      .map_err(|_| anyhow!("The clipboard thread panicked"))?
      .context("Failed to copy to the clipboard")
  }
}
