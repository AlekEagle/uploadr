use std::fs::*;
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NotificationConfig {
  pub enabled: bool,
  pub sound: Option<String>,
  pub timeout: u32,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)] // Frick you I like camel case
pub struct ArchiveConfig {
  pub enabled: bool,
  pub histFile: String,
  pub maxCount: u32,
  pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardConfig {
  pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)] // Frick you I like camel case
pub struct PrimaryConfig {
  pub defaultUploader: String,
  pub notification: NotificationConfig,
  pub clipboard: ClipboardConfig,
  pub archive: ArchiveConfig,
}

pub struct Config {
  pub config_path: String,
  pub config: PrimaryConfig,
}

impl Config {
  // Static method to get default config path
  pub fn get_default_config_path() -> String {
    let mut path: std::path::PathBuf = env::var("HOME").unwrap().into();
    path.push(".config");
    path.push("uploadr");
    path.push("config.json");
    return path.to_str().unwrap().to_string();
  }

  // Static method for """constructor"""
  pub fn new(path: Option<String>) -> Config {
    let config_path = match path {
      Some(path) => path,
      None => Config::get_default_config_path(),
    };

    let config = match Config::read_config(&config_path) {
      Ok(config) => config,
      Err(e) => panic!("Could not read config file: {}", e),
    };

    Config {
      config_path,
      config,
    }
  }

  // Static method to read config file
  pub fn read_config(path: &str) -> Result<PrimaryConfig, std::io::Error> {
    let file = File::open(path)?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
  }
}