use std::{io::Read, path::PathBuf};
use infer::Infer;

/// A File struct to hold the file data.
#[derive(Debug, Clone)]
pub struct File {
  pub name: String,
  pub buffer: Vec<u8>,
  pub mime: String,
  pub ext: String,
}

impl File {
  pub fn new(buf: &[u8], name: String) -> Self {
    let infer = Infer::new().get(buf);
    let mime = match infer {
      Some(infer) => infer.mime_type().to_owned(),
      None => if buf.is_ascii() { "text/plain".to_owned() } else { "application/octet-stream".to_owned() },
    };
    let ext = match infer {
      Some(infer) => infer.extension().to_owned(),
      None => if mime == "text/plain" { "txt".to_owned() } else { "bin".to_owned() },
    };
    File {
      name: format!("{}.{}", name, ext),
      buffer: buf.to_vec(),
      mime,
      ext,
    }
  }

  pub fn from_stdin() -> Self {
    // Check if stdin is a terminal.
    if atty::is(atty::Stream::Stdin) {
      panic!("No file specified and stdin is interactive.");
    }
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(buf.as_mut()).unwrap();
    File::new(&buf, "stdin".to_owned())
  }

  pub fn from_path(path: &PathBuf) -> Self {
    // Get the file name from the path.
    let name = path.file_name().unwrap().to_str().unwrap().to_owned();
    // Read the file into a buffer.
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(buf.as_mut()).unwrap();
    File::new(&buf, name)
  }

  pub fn override_name(&mut self, name: String) {
    self.name = name;
    // Get the file extension from the name.
    self.ext = PathBuf::from(&self.name).extension().unwrap().to_str().unwrap().to_owned();
  }

  pub fn override_mime(&mut self, mime: String) {
    self.mime = mime;
  }
}