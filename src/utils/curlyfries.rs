use crate::utils::file::File;
use anyhow::{bail, Context, Result};
use curl::easy::{Easy, List, ReadError};
use std::collections::HashMap;
use std::os::raw::c_long;
use std::time::Duration;

use super::config::{Body, Config};

/// Maximum number of redirects to follow.
const MAX_REDIRECTIONS: u32 = 5;
/// How long to wait for a connection to be established.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Abort the transfer if it's slower than LOW_SPEED_LIMIT bytes/sec for LOW_SPEED_TIME.
const LOW_SPEED_LIMIT: u32 = 1;
const LOW_SPEED_TIME: Duration = Duration::from_secs(60);

pub struct Response {
  pub status_code: u32,
  pub url: String,
  /// Headers of the final response, names are lowercase.
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
  pub response: Option<Response>,
}

impl CurlyFry {
  pub fn new() -> Result<Self> {
    let mut easy = Easy::new();
    easy.follow_location(true)?;
    easy.max_redirections(MAX_REDIRECTIONS)?;
    // Only allow HTTP(S), for the initial request and for redirects.
    restrict_protocols(&easy)?;
    easy.connect_timeout(CONNECT_TIMEOUT)?;
    easy.low_speed_limit(LOW_SPEED_LIMIT)?;
    easy.low_speed_time(LOW_SPEED_TIME)?;
    easy.useragent(format!("Uploadr/{}", env!("CARGO_PKG_VERSION")).as_str())?;
    Ok(CurlyFry {
      easy,
      url: String::new(),
      method: String::new(),
      headers: HashMap::new(),
      response: None,
    })
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

  pub fn execute(&mut self, config: &Config, file: &File) -> Result<()> {
    self.easy.url(&self.url)?;
    self.easy.custom_request(&self.method)?;

    let raw_body = match &config.uploader.request.body {
      Body::MultipartFormData { field } => {
        let mut form = curl::easy::Form::new();
        let mut part = form.part(field.as_str());
        // curl needs to own the data for the form, so this is the one copy of the file we make.
        part.buffer(&file.name, file.buffer.clone());
        part.content_type(file.mime.as_str());
        part.add().context("Failed to build the multipart form")?;
        self.easy.httppost(form)?;
        None
      }
      Body::Raw => {
        self.easy.post(true)?;
        self.easy.post_field_size(file.buffer.len() as u64)?;
        // The file's MIME type is the Content-Type, replace any configured one. (Header names are case-insensitive)
        self
          .headers
          .retain(|key, _| !key.eq_ignore_ascii_case("content-type"));
        self
          .headers
          .insert("Content-Type".to_string(), file.mime.clone());
        // The body is streamed from the file buffer in the read function below.
        Some(file.buffer.as_slice())
      }
    };

    let mut headers = List::new();
    for (key, value) in &self.headers {
      headers.append(&format_header(key, value)?)?;
    }
    self.easy.http_headers(headers)?;

    let mut response_body = Vec::new();
    let mut response_headers: HashMap<String, String> = HashMap::new();
    {
      let is_raw = raw_body.is_some();
      let mut raw_body = raw_body.unwrap_or_default();
      let mut transfer = self.easy.transfer();
      if is_raw {
        transfer.read_function(|into| {
          let len = into.len().min(raw_body.len());
          into[..len].copy_from_slice(&raw_body[..len]);
          raw_body = &raw_body[len..];
          Ok::<usize, ReadError>(len)
        })?;
      }
      transfer.write_function(|data| {
        response_body.extend_from_slice(data);
        Ok(data.len())
      })?;
      transfer.header_function(|data| {
        parse_header_line(&mut response_headers, &String::from_utf8_lossy(data));
        true
      })?;
      transfer
        .perform()
        .with_context(|| format!("The request to {} failed", self.url))?;
    }
    let response_code = self.easy.response_code()?;
    let response_url = self
      .easy
      .effective_url()?
      .map(|url| url.to_string())
      .unwrap_or_else(|| self.url.clone());

    self.response = Some(Response {
      status_code: response_code,
      url: response_url,
      headers: response_headers,
      body: String::from_utf8_lossy(&response_body).to_string(),
    });
    Ok(())
  }
}

/// Format a request header, refusing CR/LF so templated values can't inject extra headers.
fn format_header(key: &str, value: &str) -> Result<String> {
  if key.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
    bail!(
      "The header \"{}\" contains a newline, which is not allowed.",
      key.escape_debug()
    );
  }
  Ok(format!("{}: {}", key, value))
}

/// Parse a single response header line into the header map.
/// A new status line (e.g. after a redirect or a 100 Continue) starts a new response, so the old headers are cleared.
fn parse_header_line(headers: &mut HashMap<String, String>, line: &str) {
  let line = line.trim_end_matches(['\r', '\n']);
  if line.starts_with("HTTP/") {
    headers.clear();
    return;
  }
  if let Some((key, value)) = line.split_once(':') {
    let key = key.trim();
    if !key.is_empty() {
      headers.insert(key.to_ascii_lowercase(), value.trim().to_string());
    }
  }
}

/// Restrict curl to HTTP and HTTPS. The curl crate doesn't expose these options, so set them through curl-sys.
fn restrict_protocols(easy: &Easy) -> Result<()> {
  let protocols = (curl_sys::CURLPROTO_HTTP | curl_sys::CURLPROTO_HTTPS) as c_long;
  for option in [
    curl_sys::CURLOPT_PROTOCOLS,
    curl_sys::CURLOPT_REDIR_PROTOCOLS,
  ] {
    // SAFETY: `easy.raw()` is a valid handle for the lifetime of `easy`, and both options take a long bitmask.
    let code = unsafe { curl_sys::curl_easy_setopt(easy.raw(), option, protocols) };
    if code != curl_sys::CURLE_OK {
      return Err(curl::Error::new(code)).context("Failed to restrict curl to HTTP(S)");
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(lines: &[&str]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in lines {
      parse_header_line(&mut headers, line);
    }
    headers
  }

  #[test]
  fn header_names_are_lowercase_and_trimmed() {
    let headers = parse(&[
      "HTTP/1.1 200 OK\r\n",
      "Content-Type:  application/json \r\n",
      "X-Thing:value:with:colons\r\n",
      "\r\n",
    ]);
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    assert_eq!(headers.get("x-thing").unwrap(), "value:with:colons");
    assert_eq!(headers.len(), 2);
  }

  #[test]
  fn only_final_response_headers_are_kept() {
    let headers = parse(&[
      "HTTP/1.1 302 Found\r\n",
      "Location: https://example.com/other\r\n",
      "X-Old: 1\r\n",
      "\r\n",
      "HTTP/2 200\r\n",
      "X-New: 2\r\n",
      "\r\n",
    ]);
    assert_eq!(headers.get("x-new").unwrap(), "2");
    assert!(!headers.contains_key("x-old"));
    assert!(!headers.contains_key("location"));
  }

  #[test]
  fn rejects_newlines_in_headers() {
    assert!(format_header("Authorization", "Bearer abc\r\nX-Evil: 1").is_err());
    assert!(format_header("X\nY", "1").is_err());
    assert_eq!(format_header("A", "b").unwrap(), "A: b");
  }

  #[test]
  fn curl_accepts_protocol_restriction() {
    assert!(CurlyFry::new().is_ok());
  }
}
