use anyhow::{bail, Context, Result};
use infer::Infer;
use std::io::{IsTerminal, Read};
use std::path::Path;

/// A File struct to hold the file data.
#[derive(Debug, Clone)]
pub struct File {
  pub name: String,
  pub buffer: Vec<u8>,
  pub mime: String,
  pub ext: String,
}

impl File {
  pub fn new(buf: Vec<u8>, name: String) -> Self {
    let infer = Infer::new().get(&buf);
    let mime = match infer {
      Some(infer) => infer.mime_type().to_owned(),
      None => {
        if std::str::from_utf8(&buf).is_ok() {
          "text/plain".to_owned()
        } else {
          "application/octet-stream".to_owned()
        }
      }
    };
    let inferred_ext = match infer {
      Some(infer) => infer.extension().to_owned(),
      None => {
        if mime == "text/plain" {
          "txt".to_owned()
        } else {
          "bin".to_owned()
        }
      }
    };
    // Keep the name (and extension) as-is if it already has an extension, otherwise add the inferred extension.
    let (name, ext) = match extension_of(&name) {
      Some(ext) => (name, ext),
      None => (format!("{}.{}", name, inferred_ext), inferred_ext),
    };
    File {
      name,
      buffer: buf,
      mime,
      ext,
    }
  }

  pub fn from_stdin() -> Result<Self> {
    // Check if stdin is a terminal.
    if std::io::stdin().is_terminal() {
      bail!("No file specified and stdin is interactive.");
    }
    let mut buf = Vec::new();
    std::io::stdin()
      .read_to_end(&mut buf)
      .context("Failed to read from stdin")?;
    // If the buffer is empty, exit.
    if buf.is_empty() {
      bail!("No data was read from stdin.");
    }
    Ok(File::new(buf, "stdin".to_owned()))
  }

  pub fn from_path(path: &Path) -> Result<Self> {
    // Get the file name from the path.
    let name = path
      .file_name()
      .with_context(|| format!("\"{}\" is not a file", path.display()))?
      .to_string_lossy()
      .into_owned();
    // Read the file into a buffer.
    let buf =
      std::fs::read(path).with_context(|| format!("Failed to read \"{}\"", path.display()))?;
    Ok(File::new(buf, name))
  }

  pub fn override_name(&mut self, name: String) {
    // Get the file extension from the name, keep the inferred extension if it doesn't have one.
    if let Some(ext) = extension_of(&name) {
      self.ext = ext;
    }
    self.name = name;
  }

  pub fn override_mime(&mut self, mime: String) {
    self.mime = mime;
  }
}

fn extension_of(name: &str) -> Option<String> {
  Path::new(name)
    .extension()
    .map(|ext| ext.to_string_lossy().into_owned())
    .filter(|ext| !ext.is_empty())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keeps_existing_extension() {
    let file = File::new(b"hello".to_vec(), "notes.md".to_owned());
    assert_eq!(file.name, "notes.md");
    assert_eq!(file.ext, "md");
    assert_eq!(file.mime, "text/plain");
  }

  #[test]
  fn adds_inferred_extension_when_missing() {
    let file = File::new(b"hello".to_vec(), "stdin".to_owned());
    assert_eq!(file.name, "stdin.txt");
    assert_eq!(file.ext, "txt");
    let file = File::new(vec![0xff, 0xfe, 0x00, 0x81], "Clipboard".to_owned());
    assert_eq!(file.name, "Clipboard.bin");
    assert_eq!(file.mime, "application/octet-stream");
  }

  #[test]
  fn utf8_is_text() {
    let file = File::new("héllo wörld".as_bytes().to_vec(), "stdin".to_owned());
    assert_eq!(file.mime, "text/plain");
  }

  #[test]
  fn override_name_without_extension() {
    let mut file = File::new(b"hello".to_vec(), "stdin".to_owned());
    file.override_name("README".to_owned());
    assert_eq!(file.name, "README");
    assert_eq!(file.ext, "txt");
    file.override_name("thing.log".to_owned());
    assert_eq!(file.ext, "log");
  }
}
