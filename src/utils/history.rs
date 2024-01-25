use std::fs::File;
use std::{fs::create_dir_all, str::FromStr};
use std::path::PathBuf;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::utils::config::Config;
use chrono::Local;


// A struct to hold the history of uploads.
#[derive(Serialize, Deserialize, Debug)]
pub struct History {
  pub file_name: String,
  pub url: String,
  pub manage_url: Option<String>,
  pub thumbnail_url: Option<String>,
}

pub struct HistoryManager {
  pub history: Vec<History>,
  pub history_path: PathBuf,
  pub max_history: u32,
}

impl HistoryManager {
  pub fn new(config: &Config) -> Self {
    let history_path = PathBuf::from_str(&config.data.archive.path).unwrap();
    // Create the history directory if it doesn't exist.
    let history_dir = history_path.clone().join("uploads");
    if !history_dir.exists() {
      create_dir_all(history_dir).unwrap();
    }
    HistoryManager {
      history: Vec::new(),
      history_path,
      max_history: config.data.archive.max,
    }
  }

  pub fn load(&mut self) -> bool {
    let history_file = self.history_path.clone().join("history.csv");
    if history_file.exists() {
      let mut reader = csv::Reader::from_path(history_file).unwrap();
      for result in reader.deserialize() {
        let record: History = result.unwrap();
        self.history.push(record);
      }
       true
    } else {
      false
    }
  }

  pub fn save(&self) {
    let history_file = self.history_path.clone().join("history.csv");
    let mut writer = csv::Writer::from_path(history_file).unwrap();
    for record in &self.history {
      writer.serialize(record).unwrap();
    }
    writer.flush().unwrap();
  }

  // Delete oldest history record.
  pub fn delete_oldest(&mut self) {
    if !self.history.is_empty() {
      let file = self.history.remove(0);
      let file_path = self.history_path.clone().join("uploads").join(file.file_name);
      std::fs::remove_file(file_path).unwrap();
      self.save();
    }
  }

  // Take a file buffer, pertinent URLs and commit them to the history.
  pub fn add(&mut self, file_buf: &super::file::File, url: String, manage_url: Option<String>, thumbnail_url: Option<String>) {
    // Check if we have reached the maximum number of history records.
    if self.history.len() >= self.max_history as usize{
      // Remove the oldest record.
      self.delete_oldest();
    }
    // Get the current time. This will be used as the filename.
    let time_str = Local::now().format("%Y-%m-%d %H-%M-%S").to_string();
    let file_name = format!("{}.{}", time_str, file_buf.ext);
    // Create the file path.
    let file_path = self.history_path.clone().join("uploads").join(&file_name);
    // Write the file.
    let mut file = File::create(file_path).unwrap();
    file.write_all(&file_buf.buffer).unwrap();
    // Create the history record.
    let history_record = History {
      file_name,
      url,
      manage_url,
      thumbnail_url,
    };
    // Add the history record to the history.
    self.history.push(history_record);
    // Commit the history to disk.
    self.save();
  }

  pub fn list(&self) -> String {
    let mut output = String::new();
    for record in &self.history {
      output.push_str(&format!("{}, {}, {}, {}\n", record.file_name, record.url, record.manage_url.as_ref().unwrap_or(&String::from("None")), record.thumbnail_url.as_ref().unwrap_or(&String::from("None"))));
    }
    // Remove the trailing newline.
    output.pop();
    output
  }

}