mod utils;
use anyhow::{bail, Context, Result};
use clap::Parser;
use notify_rust::Notification;
use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;
use utils::clippy::Clippy;
use utils::config::Config;
use utils::curlyfries::CurlyFry;
use utils::file::File;
use utils::history::HistoryManager;
use utils::templator::Templator;

#[derive(Parser, Debug)]
#[command(author = "Alek Evans", version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Args {
  /// The path to the config directory to use (contains config.json and the .uploader files).
  #[clap(short, long)]
  pub config: Option<String>,

  /// The uploader to use.
  #[clap(short, long)]
  pub uploader: Option<String>,

  /// Displays upload history.
  #[clap(short = 'H', long)]
  pub history: bool,

  /// Forcefully specify the filename.
  #[clap(short, long)]
  pub filename: Option<String>,

  /// Forcefully specify the mime type.
  #[clap(short, long)]
  pub mime: Option<String>,

  /// The file to upload. If not specified or set to "-", stdin will be used, "@" will use the clipboard.
  pub file: Option<String>,
}

fn main() -> ExitCode {
  env_logger::init();
  match run() {
    Ok(code) => code,
    Err(err) => {
      eprintln!("Error: {:#}", err);
      ExitCode::FAILURE
    }
  }
}

fn run() -> Result<ExitCode> {
  let mut args: Args = Args::parse();
  let config = Config::new(args.config.take(), args.uploader.take())?;
  let mut templator = Templator::new(&config);

  // Print history if requested and exit.
  if args.history {
    return print_history(&config, HistoryManager::new(&config));
  }

  let mut file = match args.file.as_deref().unwrap_or("-") {
    "-" => File::from_stdin()?,
    "@" => {
      if !config.data.clipboard.enabled {
        bail!("Clipboard is not enabled, can't upload from the clipboard.");
      }
      Clippy::new()?.get()?
    }
    file => File::from_path(Path::new(file))?,
  };

  if let Some(filename) = args.filename {
    file.override_name(filename);
  }

  if let Some(mime) = args.mime {
    file.override_mime(mime);
  }

  templator.set_file(&file);

  let mut curlyfry = CurlyFry::new()?;
  curlyfry.set_url(templator.format(&config.uploader.request.url)?);
  curlyfry.set_method(config.uploader.request.method.clone());
  let mut headers = HashMap::new();
  for (key, value) in &config.uploader.request.headers {
    headers.insert(key.clone(), templator.format(value)?);
  }
  curlyfry.set_headers(headers);
  let progress_notification = if config.data.notification.enabled {
    Notification::new()
      .appname("Uploadr")
      .summary("Uploading...")
      .timeout(0)
      .body(&format!("Uploading {}...", file.name))
      .show()
      .map_err(|err| eprintln!("Failed to show notification: {}", err))
      .ok()
  } else {
    None
  };
  let result = curlyfry.execute(&config, &file);
  // Always close the progress notification, even if the upload failed.
  if let Some(notification) = progress_notification {
    notification.close();
  }
  if let Err(err) = result {
    println!("Upload failed!");
    notify_failure(&config, &format!("{:#}", err));
    return Err(err);
  }

  templator.set_curlyfry(&curlyfry);
  let response = curlyfry
    .response
    .as_ref()
    .context("No response was received.")?;

  if response.ok() {
    println!("Upload successful!");
    let url = templator.format(&config.uploader.response.url)?;
    let manage_url = config
      .uploader
      .response
      .manage_url
      .as_ref()
      .map(|manage_url| templator.format(manage_url))
      .transpose()?;
    let thumbnail_url = config
      .uploader
      .response
      .thumbnail_url
      .as_ref()
      .map(|thumbnail_url| templator.format(thumbnail_url))
      .transpose()?;
    // Print and copy the link before anything else can go wrong.
    println!("URL: {}", url);
    if let Some(manage_url) = &manage_url {
      println!("Manage URL: {}", manage_url);
    }
    if let Some(thumbnail_url) = &thumbnail_url {
      println!("Thumbnail URL: {}", thumbnail_url);
    }
    let clipboard_write = if !config.data.clipboard.enabled {
      None
    } else if config.data.clipboard.read_only {
      println!("Clipboard is read-only, not copying URL to clipboard.");
      None
    } else {
      Some(Clippy::set(url.clone()))
    };
    if config.data.notification.enabled {
      notify_success(&config, url.clone(), manage_url.clone());
    }
    if config.data.archive.enabled {
      if let Err(err) = archive(&config, &file, url, manage_url, thumbnail_url) {
        eprintln!("Failed to save upload history: {:#}", err);
      }
    }
    if let Some(clipboard_write) = clipboard_write {
      if let Err(err) = Clippy::wait(clipboard_write) {
        eprintln!("{:#}", err);
      }
    }
    Ok(ExitCode::SUCCESS)
  } else {
    println!("Upload failed!");
    println!("Status Code: {}", response.status_code);
    println!("Response: {}", response.body);
    notify_failure(
      &config,
      &format!(
        "Status Code: {}\nResponse: {}",
        response.status_code, response.body
      ),
    );
    Ok(ExitCode::FAILURE)
  }
}

fn archive(
  config: &Config,
  file: &File,
  url: String,
  manage_url: Option<String>,
  thumbnail_url: Option<String>,
) -> Result<()> {
  let mut history_manager = HistoryManager::new(config);
  history_manager.load()?;
  history_manager.add(file, url, manage_url, thumbnail_url)
}

/// Only open http(s) URLs in the browser.
fn open_url(url: &str) {
  let lower = url.to_ascii_lowercase();
  if !(lower.starts_with("http://") || lower.starts_with("https://")) {
    eprintln!("Not opening \"{}\", it is not an http(s) URL.", url);
    return;
  }
  if let Err(err) = webbrowser::open(url) {
    eprintln!("Failed to open \"{}\": {}", url, err);
  }
}

fn notify_success(config: &Config, url: String, manage_url: Option<String>) {
  let timeout =
    i32::try_from(config.data.notification.timeout.saturating_mul(1000)).unwrap_or(i32::MAX);
  // spawn a thread to display the notification
  std::thread::spawn(move || {
    let mut notification = Notification::new();
    notification
      .appname("Uploadr")
      .summary("Upload successful!")
      .body("Click to open in your browser.")
      .timeout(timeout)
      .action("open", "Open")
      .action("default", "Open");
    if manage_url.is_some() {
      notification.action("manage", "Manage Upload");
    }
    match notification.show() {
      Ok(handle) => handle.wait_for_action(|action| match action {
        "open" | "default" => open_url(&url),
        "manage" => {
          if let Some(manage_url) = &manage_url {
            open_url(manage_url);
          }
        }
        _ => {}
      }),
      Err(err) => eprintln!("Failed to show notification: {}", err),
    }
  });
}

fn notify_failure(config: &Config, body: &str) {
  if !config.data.notification.enabled {
    return;
  }
  if let Err(err) = Notification::new()
    .appname("Uploadr")
    .summary("Upload failed!")
    .timeout(0)
    .body(body)
    .show()
  {
    eprintln!("Failed to show notification: {}", err);
  }
}

fn print_history(config: &Config, mut history_manager: HistoryManager) -> Result<ExitCode> {
  if config.data.archive.enabled {
    if history_manager.load()? {
      let history = history_manager.list();
      println!("File Name, URL, Manage URL, Thumbnail URL");
      println!("----------------");
      println!("{history}");
      Ok(ExitCode::SUCCESS)
    } else {
      println!("No upload history found.");
      Ok(ExitCode::FAILURE)
    }
  } else {
    println!("Upload history is disabled.");
    Ok(ExitCode::FAILURE)
  }
}
