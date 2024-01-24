use arboard::{Clipboard, SetExtLinux};
use image::{DynamicImage, ImageBuffer};
use super::file::File;

pub struct Clippy {
  pub clipboard: Clipboard,
}

impl Clippy {
  pub fn new() -> Self {
    Clippy {
      clipboard: Clipboard::new().unwrap(),
    }
  }

  pub fn get(&mut self) -> Result<File, String> {
    let image = self.clipboard.get_image();
    let text = self.clipboard.get_text();
    match (image, text) {
      (Ok(image), _) => {
        // Get the raw image data. (Width, Height, pixel values)
        let img_buf = ImageBuffer::from_raw(image.width as u32, image.height as u32, image.bytes.to_vec()).unwrap();
        // Convert the raw image data into a DynamicImage.
        let img = DynamicImage::ImageRgba8(img_buf);
        // Create a buffer to hold the image data.
        let mut buf = Vec::new();
        // Create a writer to write the image data to the buffer.
        let mut writer = std::io::Cursor::new(&mut buf);
        // Write the image data to the buffer.
        img.write_to(&mut writer, image::ImageOutputFormat::Png).unwrap();
        // Create a new file from the buffer.
        Ok(File::new(&buf, "Clipboard".to_owned()))
      },
      (_, Ok(text)) => Ok(File::new(text.as_bytes(), "Clipboard".to_owned())),
      _ => Err("No image or text in clipboard. Or clipboard does not contain supported format.".to_owned()),
    }
  }

  pub fn set(&self, data: String) {
    // Create a new thread so we can wait for the clipboard to be set without blocking the main thread.
    std::thread::spawn(move || {
      Clipboard::new()?.set().wait().text(data)?;
      Ok::<(), arboard::Error>(())
    }).join().unwrap().unwrap();
  }
}