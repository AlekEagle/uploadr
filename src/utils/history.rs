use super::private_fs;
use crate::utils::config::Config;
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use uuid::Uuid;

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
    // The history directory is only created once something is archived.
    HistoryManager {
      history: Vec::new(),
      history_path: PathBuf::from(&config.data.archive.path),
      max_history: config.data.archive.max,
    }
  }

  fn history_file(&self) -> PathBuf {
    self.history_path.join("history.csv")
  }

  fn uploads_dir(&self) -> PathBuf {
    self.history_path.join("uploads")
  }

  pub fn load(&mut self) -> Result<bool> {
    let history_file = self.history_file();
    if history_file.exists() {
      let mut reader = csv::Reader::from_path(&history_file)
        .with_context(|| format!("Failed to open {}", history_file.display()))?;
      for (index, result) in reader.deserialize().enumerate() {
        match result {
          Ok(record) => self.history.push(record),
          // Skip broken rows instead of failing the whole history.
          Err(err) => eprintln!(
            "Skipping invalid history record {} in {}: {}",
            index + 1,
            history_file.display(),
            err
          ),
        }
      }
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn save(&self) -> Result<()> {
    let history_file = self.history_file();
    private_fs::create_dir_all(&self.history_path)
      .with_context(|| format!("Failed to create {}", self.history_path.display()))?;
    let file = private_fs::create_file(&history_file)
      .with_context(|| format!("Failed to create {}", history_file.display()))?;
    let mut writer = csv::Writer::from_writer(file);
    for record in &self.history {
      writer.serialize(record)?;
    }
    writer
      .flush()
      .with_context(|| format!("Failed to write {}", history_file.display()))
  }

  // Delete oldest history record.
  pub fn delete_oldest(&mut self) -> Result<()> {
    if !self.history.is_empty() {
      let file = self.history.remove(0);
      let file_path = self.uploads_dir().join(file.file_name);
      match std::fs::remove_file(&file_path) {
        Ok(()) => {}
        // Already gone, nothing to do.
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
          return Err(err).with_context(|| format!("Failed to delete {}", file_path.display()))
        }
      }
      self.save()?;
    }
    Ok(())
  }

  // Take a file buffer, pertinent URLs and commit them to the history.
  pub fn add(
    &mut self,
    file_buf: &super::file::File,
    url: String,
    manage_url: Option<String>,
    thumbnail_url: Option<String>,
  ) -> Result<()> {
    // Remove the oldest records until we're below the maximum number of history records.
    while !self.history.is_empty() && self.history.len() >= self.max_history as usize {
      self.delete_oldest()?;
    }
    let uploads_dir = self.uploads_dir();
    private_fs::create_dir_all(&uploads_dir)
      .with_context(|| format!("Failed to create {}", uploads_dir.display()))?;
    // Get the current time (with milliseconds). This will be used as the filename.
    let time_str = Local::now().format("%Y-%m-%d %H-%M-%S%.3f").to_string();
    let mut file_name = format!("{}.{}", time_str, file_buf.ext);
    // Write the file, never overwriting an existing archived file.
    let mut file = match private_fs::create_new_file(&uploads_dir.join(&file_name)) {
      Err(err) if err.kind() == ErrorKind::AlreadyExists => {
        file_name = format!("{} {}.{}", time_str, Uuid::new_v4(), file_buf.ext);
        private_fs::create_new_file(&uploads_dir.join(&file_name))
      }
      result => result,
    }
    .with_context(|| format!("Failed to create the archived file {}", file_name))?;
    file
      .write_all(&file_buf.buffer)
      .with_context(|| format!("Failed to write the archived file {}", file_name))?;
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
    self.save()
  }

  pub fn list(&self) -> String {
    let mut output = String::new();
    for record in &self.history {
      output.push_str(&format!(
        "{}, {}, {}, {}\n",
        record.file_name,
        record.url,
        record.manage_url.as_deref().unwrap_or("None"),
        record.thumbnail_url.as_deref().unwrap_or("None")
      ));
    }
    // Remove the trailing newline.
    output.pop();
    output
  }
}
