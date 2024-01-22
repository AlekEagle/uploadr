use std::{io::Read, path::PathBuf};
use curl::easy::Easy;
use infer::Infer;
use crate::utils::config::UploaderData;

/// A File struct to hold the file data.
pub struct File {
  pub name: Option<String>,
  pub buffer: Vec<u8>,
  pub mime: String,
  pub ext: String,
}

impl File {
  pub fn new(buf: &[u8], name: Option<String>) -> Self {
    let infer = Infer::new().get(&buf);
    let mime = match infer {
      Some(infer) => infer.mime_type().to_owned(),
      None => "application/octet-stream".to_owned(),
    };
    let ext = match infer {
      Some(infer) => infer.extension().to_owned(),
      None => "bin".to_owned(),
    };
    return File {
      name,
      buffer: buf.to_vec(),
      mime,
      ext,
    };
  }

  pub fn from_file(path: &PathBuf) -> Self {
    // Get the file name from the path.
    let name = path.file_name().unwrap().to_str().unwrap().to_owned();
    // Read the file into a buffer.
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(buf.as_mut()).unwrap();
    return File::new(&buf, Some(name));
  }
}

pub struct CurlyFry<'a> {
  pub easy: Easy,
  pub uploader: &'a UploaderData,
}

impl<'a> CurlyFry<'a> {
  pub fn new(uploader: &'a UploaderData) -> Self {
    return CurlyFry {
      easy: Easy::new(),
      uploader,
    };
  }

  fn choose_method(&mut self) -> () {
    match self.uploader.request.method.as_str() {
      "POST" => self.easy.post(true).unwrap(),
      "PUT" => self.easy.put(true).unwrap(),
      _ => panic!("Uploader config contains an invalid method. {}", self.uploader.request.method),
    };
  }

  fn set_headers(&mut self) -> () {
    let mut list = curl::easy::List::new();
    for (key, value) in &self.uploader.request.headers {
      list.append(&format!("{}: {}", key, value)).unwrap();
    }
  }
}