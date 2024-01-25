use std::collections::HashMap;
use curl::easy::{Easy, List};
use crate::utils::file::File;

use super::config::{Body, Config};

pub struct Response {
  pub status_code: u32,
  pub url: String,
  pub headers: HashMap<String, String>,
  pub body: String,
}

impl Response {
  pub fn ok(&self) -> bool {
    self.status_code / 100 == 2
  }
}

pub struct CurlyFry {
  pub easy: Easy,
  pub url: String,
  pub method: String,
  pub headers: HashMap<String, String>,
  pub file: File,
  pub response: Option<Response>,
}

impl CurlyFry {
  pub fn new() -> Self {
    let mut easy = Easy::new();
    easy.follow_location(true).unwrap();
    easy.useragent(format!("Uploadr/{}", env!("CARGO_PKG_VERSION")).as_str()).unwrap();
    CurlyFry {
      easy,
      url: String::new(),
      method: String::new(),
      headers: HashMap::new(),
      file: File::new(&[], "This will be overridden".to_owned()),
      response: None,
    }
  }

  pub fn set_url(&mut self, url: String) {
    self.url = url;
  }

  pub fn set_method(&mut self, method: String) {
    self.method = method;
  }

  pub fn set_headers(&mut self, headers: HashMap<String, String>) {
    self.headers = headers;
  }

  pub fn set_file(&mut self, file: File) {
    self.file = file;
  }

  pub fn execute(&mut self, config: &Config) {
    self.easy.url(&self.url).unwrap();
    self.easy.custom_request(&self.method).unwrap();
    
    match &config.uploader.request.body {
      Body::MultipartFormData { field } => {
        let mut form = curl::easy::Form::new();
        let mut part = form.part(field.as_str());
        let filename = self.file.name.clone();
        let mime = self.file.mime.as_str();
        part.buffer(&filename, self.file.buffer.clone());
        part.content_type(mime);
        part.add().unwrap();
        self.easy.httppost(form).unwrap();
      }
      Body::Raw => {
        self.easy.post(true).unwrap();
        self.easy.post_field_size(self.file.buffer.len() as u64).unwrap();
        self.easy.post_fields_copy(self.file.buffer.as_slice()).unwrap();
        self.headers.insert("Content-Type".to_string(), self.file.mime.clone());
      }
    }

    let mut headers = List::new();
    for (key, value) in &self.headers {
      headers.append(&format!("{}: {}", key, value)).unwrap();
    }
    self.easy.http_headers(headers).unwrap();

    let mut response_body = Vec::new();
    let mut response_headers: HashMap<String, String> = HashMap::new();
    {
      let mut transfer = self.easy.transfer();
      transfer.write_function(|data| {
        response_body.extend_from_slice(data);
        Ok(data.len())
      }).unwrap();
      transfer.header_function(|data| {
        let header = String::from_utf8_lossy(data).to_string();
        if !header.contains(": ") {
          return true;
        }
        let mut parts = header.splitn(2, ": ");
        let key = parts.next().unwrap().to_string();
        let value = parts.next().unwrap().to_string();
        response_headers.insert(key, value);
        true
      }).unwrap();
      transfer.perform().unwrap();
    }
    let response_code = self.easy.response_code().unwrap();
    let response_url = self.easy.effective_url().unwrap();

    self.response = Some(Response {
      status_code: response_code,
      url: response_url.unwrap().to_string(),
      headers: response_headers,
      body: String::from_utf8_lossy(&response_body).to_string(),
    });
  }
}