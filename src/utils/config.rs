use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::fs::*;
use std::env;
use serde::{Deserialize, Serialize};

/// The all-encompassing config struct.
#[derive(Debug)]
pub struct Config {
  pub config_path: PathBuf,
  pub data: ConfigData,
  pub uploader: UploaderData,
}

/// The data struct.
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigData {
  // Can be camel case in the config file.
  #[serde(rename = "defaultUploader")]
  pub default_uploader: String,
  pub notification: Notification,
  pub archive: Archive,
  pub clipboard: Clipboard,
}

/// Notification config struct.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
  pub enabled: bool,
  pub timeout: u32,
}

/// Archive config struct.
#[derive(Serialize, Deserialize, Debug)]
pub struct Archive {
  pub enabled: bool,
  pub path: String,
  #[serde(rename = "maxCount")]
  pub max: u32,
}

/// Clipboard config struct.
#[derive(Serialize, Deserialize, Debug)]
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
  pub method: String,
  pub headers: HashMap<String, String>,
  pub body: Body
}

/// Struct for body data. Only one can be specified at a time.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Body {
  Raw,
  MultipartFormData {
    field: String,
  },
}

/// Struct for the response data.
/// URL must be provided from the response.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploaderResponse {
  pub url: String,
  pub manage_url: Option<String>,
  pub thumbnail_url: Option<String>,
}

impl Config {
  /// A static method to get the default config directory.
  pub fn get_default_config_path() -> PathBuf {
     PathBuf::from_iter(&[env::var("HOME").unwrap(), ".config".to_owned(), "uploadr".to_owned()])
  }

  pub fn resolve_path(path: &str) -> PathBuf {
    let mut resolved_path: PathBuf;
    // If the path is absolute, return it.
    if PathBuf::from(path).is_absolute() {
      return PathBuf::from(path);
    } else {
      // If the path starts with "~", resolve it.
      if path.starts_with("~/") {
        resolved_path = PathBuf::from(env::var("HOME").unwrap());
        resolved_path.push(path.trim_start_matches("~/"));
        return resolved_path;
      } else {
        // Otherwise, resolve it relative to the default config directory.
        resolved_path = Config::get_default_config_path();
        resolved_path.push(path);
      }
    }
    // If the parent directory doesn't exist, panic.
    if !resolved_path.parent().unwrap().exists() {
      panic!("The parent directory of the path \"{}\" does not exist.", path);
    }
    // Return the resolved path.
    resolved_path
  }

  /// A static method to get the default config.
  pub fn new(config_path: Option<String>, uploader: Option<String>) -> Self {
    let config_path = match config_path {
      Some(config_path) => Config::resolve_path(&config_path),
      None => Config::get_default_config_path(),
    };
    let data = ConfigData::from_file(&config_path);
    let uploader = UploaderData::from_file(&config_path, &uploader.unwrap_or(data.default_uploader.clone()));
    Config {
      config_path,
      data,
      uploader,
    }
  }
}

impl ConfigData {
  /// A static method that returns default ConfigData.
  pub fn default() -> Self {
    ConfigData {
      default_uploader: "imgur".to_owned(),
      notification: Notification {
        enabled: true,
        timeout: 5,
      },
      archive: Archive {
        enabled: true,
        path: PathBuf::from_iter(&[env::var("HOME").unwrap(), "Pictures".to_owned(), "uploadr".to_owned()]).to_str().unwrap().to_string(),
        max: 100,
      },
      clipboard: Clipboard {
        enabled: true,
        read_only: false,
      },
    }
  }

  pub fn from_file(config_path: &Path) -> Self {
    // Clone the path buf so we can modify it without affecting the original.
    let mut config_path = config_path.to_path_buf();
    // Append the config file name to the path.
    config_path.push("config.json");
    // Create the config file if it doesn't exist.
    if !config_path.exists() {
      create_config_dir(&config_path);
      let default_config = ConfigData::default();
      default_config.save(&config_path);
    }
    let file = File::open(config_path).unwrap();
    // Read from reader.
    match serde_json::from_reader(file) {
      Ok(config) => config,
      Err(_) => {
        panic!("The config file is invalid. Please fix it or delete it and run the program again.");
      }
    }
  }

  fn save(&self, path: &Path) {
    let mut file = File::create(path).unwrap();
    let config_json = self.stringify();
    file.write_all(config_json.as_bytes()).unwrap();
  }

  fn stringify(&self) -> String {
    serde_json::to_string_pretty(&self).unwrap()
  }

  pub fn to_value(&self) -> serde_json::Value {
    serde_json::to_value(self).unwrap()
  }
}

impl UploaderData {
  /// A static method that returns default UploaderData. (An Imgur uploader config)
  pub fn default() -> Self {
    UploaderData {
      request: UploaderRequest {
        url: "https://api.imgur.com/3/image".to_owned(),
        method: "POST".to_owned(),
        headers: {
          let mut map = HashMap::new();
          map.insert("Authorization".to_owned(), "Client-ID eb13b71463957f7".to_owned());
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

  pub fn from_file(config_path: &Path, uploader: &str) -> Self {
    // Clone the path buf so we can modify it without affecting the original.
    let mut config_path = config_path.to_path_buf();
    // Append the config file name to the path.
    config_path.push(format!("{}.uploader", uploader));
    // Create the "imgur.uploader" file if it doesn't exist. 
    // If it is not "imgur", then we should panic because the user is (probably) trying to use a custom uploader that doesn't exist that they think should exist.
    if !config_path.exists() && uploader != "imgur" {
      panic!("The uploader \"{}\" does not exist.", uploader);
    } else if !config_path.exists() {
      create_config_dir(&config_path);
      let default_config = UploaderData::default();
      default_config.save(&config_path);
    }
    let file = File::open(config_path).unwrap();
    // Read from reader.
    match serde_json::from_reader(file) {
      Ok(config) => config,
      Err(_) => {
        panic!("The uploader {} is invalid. Please fix it or delete it and run the program again.", uploader);
      }
    }
  }

  fn save(&self, path: &Path) {
    let mut file = File::create(path).unwrap();
    let config_json = self.stringify();
    file.write_all(config_json.as_bytes()).unwrap();
  }

  fn stringify(&self) -> String {
    serde_json::to_string_pretty(&self).unwrap()
  }

  pub fn to_value(&self) -> serde_json::Value {
    serde_json::to_value(self).unwrap()
  }
}

fn create_config_dir(config_path: &Path) {
  let mut config_path = config_path.to_path_buf();
  config_path.pop();
  create_dir_all(config_path).unwrap();
}