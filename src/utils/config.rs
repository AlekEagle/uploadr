use super::private_fs;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// The default Imgur Client-ID, can be overridden with the UPLOADR_IMGUR_CLIENT_ID environment variable.
const DEFAULT_IMGUR_CLIENT_ID: &str = "eb13b71463957f7";

/// The all-encompassing config struct.
#[derive(Debug)]
pub struct Config {
  pub data: ConfigData,
  pub uploader: UploaderData,
}

/// The data struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigData {
  pub default_uploader: String,
  pub notification: Notification,
  pub archive: Archive,
  pub clipboard: Clipboard,
}

/// Notification config struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Notification {
  pub enabled: bool,
  pub timeout: u32,
}

/// Archive config struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Archive {
  pub enabled: bool,
  pub path: String,
  pub max: u32,
}

/// Clipboard config struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Clipboard {
  pub enabled: bool,
  pub read_only: bool,
}

/// Struct for specific uploaders.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploaderData {
  pub request: UploaderRequest,
  pub response: UploaderResponse,
}

/// Struct for the request data.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploaderRequest {
  pub url: String,
  #[serde(default = "default_method")]
  pub method: String,
  #[serde(default)]
  pub headers: HashMap<String, String>,
  pub body: Body,
}

/// Struct for body data. Only one can be specified at a time.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Body {
  Raw,
  MultipartFormData { field: String },
}

/// Struct for the response data.
/// URL must be provided from the response.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploaderResponse {
  pub url: String,
  #[serde(default)]
  pub manage_url: Option<String>,
  #[serde(default)]
  pub thumbnail_url: Option<String>,
}

fn default_method() -> String {
  "POST".to_owned()
}

/// The user's home directory, falls back to the HOME environment variable.
fn home_dir() -> Result<PathBuf> {
  dirs::home_dir()
    .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
    .context("Could not determine your home directory.")
}

impl Config {
  /// A static method to get the default config directory. ($XDG_CONFIG_HOME/uploadr, usually ~/.config/uploadr)
  pub fn get_default_config_path() -> Result<PathBuf> {
    let config_dir = match dirs::config_dir() {
      Some(config_dir) => config_dir,
      None => home_dir()?.join(".config"),
    };
    Ok(config_dir.join("uploadr"))
  }

  pub fn resolve_path(path: &str) -> Result<PathBuf> {
    let mut resolved_path: PathBuf;
    // If the path is absolute, return it.
    if PathBuf::from(path).is_absolute() {
      return Ok(PathBuf::from(path));
    } else {
      // If the path starts with "~", resolve it.
      if path.starts_with("~/") {
        resolved_path = home_dir()?;
        resolved_path.push(path.trim_start_matches("~/"));
      } else {
        // Otherwise, resolve it relative to the default config directory.
        resolved_path = Config::get_default_config_path()?;
        resolved_path.push(path);
      }
    }
    // Missing directories are created when the config is first written.
    Ok(resolved_path)
  }

  /// A static method to get the default config.
  pub fn new(config_path: Option<String>, uploader: Option<String>) -> Result<Self> {
    let config_path = match config_path {
      Some(config_path) => Config::resolve_path(&config_path)?,
      None => Config::get_default_config_path()?,
    };
    let data = ConfigData::from_file(&config_path)?;
    let uploader = UploaderData::from_file(
      &config_path,
      &uploader.unwrap_or(data.default_uploader.clone()),
    )?;
    Ok(Config { data, uploader })
  }
}

impl Default for Notification {
  fn default() -> Self {
    Notification {
      enabled: true,
      timeout: 5,
    }
  }
}

impl Default for Archive {
  fn default() -> Self {
    Archive {
      enabled: true,
      // If the home directory can't be found, this ends up relative to the working directory.
      path: home_dir()
        .unwrap_or_default()
        .join("Pictures")
        .join("uploadr")
        .to_string_lossy()
        .into_owned(),
      max: 100,
    }
  }
}

impl Default for Clipboard {
  fn default() -> Self {
    Clipboard {
      enabled: true,
      read_only: false,
    }
  }
}

impl Default for ConfigData {
  /// Returns the default ConfigData.
  fn default() -> Self {
    ConfigData {
      default_uploader: "imgur".to_owned(),
      notification: Notification::default(),
      archive: Archive::default(),
      clipboard: Clipboard::default(),
    }
  }
}

impl ConfigData {
  pub fn from_file(config_path: &Path) -> Result<Self> {
    // Clone the path buf so we can modify it without affecting the original.
    let mut config_path = config_path.to_path_buf();
    // Append the config file name to the path.
    config_path.push("config.json");
    // Create the config file if it doesn't exist.
    if !config_path.exists() {
      create_config_dir(&config_path)?;
      let default_config = ConfigData::default();
      default_config.save(&config_path)?;
    }
    let file = File::open(&config_path)
      .with_context(|| format!("Failed to open the config file {}", config_path.display()))?;
    // Read from reader.
    serde_json::from_reader(file).with_context(|| {
      format!(
        "The config file {} is invalid. Please fix it or delete it and run the program again",
        config_path.display()
      )
    })
  }

  fn save(&self, path: &Path) -> Result<()> {
    write_config(path, &self.stringify()?)
  }

  fn stringify(&self) -> Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  pub fn to_value(&self) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(self)?)
  }
}

impl Default for UploaderData {
  /// Returns the default UploaderData. (An Imgur uploader config)
  fn default() -> Self {
    UploaderData {
      request: UploaderRequest {
        url: "https://api.imgur.com/3/image".to_owned(),
        method: "POST".to_owned(),
        headers: {
          let mut map = HashMap::new();
          map.insert(
            "Authorization".to_owned(),
            format!(
              "Client-ID {{env:UPLOADR_IMGUR_CLIENT_ID;{}}}",
              DEFAULT_IMGUR_CLIENT_ID
            ),
          );
          map
        },
        body: Body::MultipartFormData {
          field: "image".to_owned(),
        },
      },
      response: UploaderResponse {
        url: "{json:$.data.link}".to_owned(),
        manage_url: None,
        thumbnail_url: None,
      },
    }
  }
}

impl UploaderData {
  pub fn from_file(config_path: &Path, uploader: &str) -> Result<Self> {
    // Clone the path buf so we can modify it without affecting the original.
    let mut config_path = config_path.to_path_buf();
    // Append the config file name to the path.
    config_path.push(format!("{}.uploader", uploader));
    // Create the "imgur.uploader" file if it doesn't exist.
    // If it is not "imgur", then we should error because the user is (probably) trying to use a custom uploader that doesn't exist that they think should exist.
    if !config_path.exists() && uploader != "imgur" {
      bail!(
        "The uploader \"{}\" does not exist. ({} not found)",
        uploader,
        config_path.display()
      );
    } else if !config_path.exists() {
      create_config_dir(&config_path)?;
      let default_config = UploaderData::default();
      default_config.save(&config_path)?;
    }
    let file = File::open(&config_path)
      .with_context(|| format!("Failed to open the uploader {}", config_path.display()))?;
    // Read from reader.
    serde_json::from_reader(file).with_context(|| {
      format!(
        "The uploader {} is invalid. Please fix it or delete it and run the program again",
        uploader
      )
    })
  }

  fn save(&self, path: &Path) -> Result<()> {
    write_config(path, &self.stringify()?)
  }

  fn stringify(&self) -> Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  pub fn to_value(&self) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(self)?)
  }
}

/// Write a config file that only the current user can read.
fn write_config(path: &Path, contents: &str) -> Result<()> {
  let mut file = private_fs::create_file(path)
    .with_context(|| format!("Failed to create {}", path.display()))?;
  file
    .write_all(contents.as_bytes())
    .with_context(|| format!("Failed to write {}", path.display()))
}

fn create_config_dir(config_path: &Path) -> Result<()> {
  let mut config_path = config_path.to_path_buf();
  config_path.pop();
  private_fs::create_dir_all(&config_path).with_context(|| {
    format!(
      "Failed to create the config directory {}",
      config_path.display()
    )
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn partial_config_uses_defaults() {
    let data: ConfigData = serde_json::from_str(
      r#"{ "default_uploader": "mine", "notification": { "enabled": false } }"#,
    )
    .unwrap();
    assert_eq!(data.default_uploader, "mine");
    assert!(!data.notification.enabled);
    assert_eq!(data.notification.timeout, 5);
    assert!(data.archive.enabled);
    assert!(data.clipboard.enabled);
  }

  #[test]
  fn existing_config_format_loads() {
    let json = serde_json::to_string_pretty(&ConfigData::default()).unwrap();
    let data: ConfigData = serde_json::from_str(&json).unwrap();
    assert_eq!(data.default_uploader, "imgur");
    let json = serde_json::to_string_pretty(&UploaderData::default()).unwrap();
    let uploader: UploaderData = serde_json::from_str(&json).unwrap();
    assert_eq!(uploader.request.method, "POST");
  }

  #[test]
  fn minimal_uploader_loads() {
    let uploader: UploaderData = serde_json::from_str(
      r#"{ "request": { "url": "https://example.com", "body": { "type": "Raw" } }, "response": { "url": "{response:url}" } }"#,
    )
    .unwrap();
    assert_eq!(uploader.request.method, "POST");
    assert!(uploader.request.headers.is_empty());
    assert!(uploader.response.manage_url.is_none());
  }

  #[cfg(unix)]
  #[test]
  fn config_files_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("uploadr-test-{}", uuid::Uuid::new_v4()));
    let config_dir = dir.join("nested").join("config");
    let data = ConfigData::from_file(&config_dir).unwrap();
    assert_eq!(data.default_uploader, "imgur");
    UploaderData::from_file(&config_dir, "imgur").unwrap();
    assert!(UploaderData::from_file(&config_dir, "missing").is_err());
    let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&config_dir), 0o700);
    assert_eq!(mode(&config_dir.join("config.json")), 0o600);
    assert_eq!(mode(&config_dir.join("imgur.uploader")), 0o600);
    std::fs::remove_dir_all(dir).unwrap();
  }
}
