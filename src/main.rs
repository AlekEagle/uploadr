mod utils;
use clap::Parser;
use notify_rust::Notification;
use std::path::PathBuf;
use std::process::ExitCode;
use utils::clippy::Clippy;
use utils::config::Config;
use utils::curlyfries::CurlyFry;
use utils::file::File;
use utils::history::HistoryManager;
use utils::templator::Templator;
use webbrowser::open;

#[derive(Parser, Debug)]
#[command(author = "Alek Evans", version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Args {
  /// The path to the config file to use.
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
  let mut args: Args = Args::parse();
  let config = Config::new(args.config.take(), args.uploader.take());
  let mut templator = Templator::new(&config);
  let mut history_manager = HistoryManager::new(&config);
  let mut clippy: Option<Clippy> = None; 

  if config.data.clipboard.enabled {
    clippy = Some(Clippy::new());
  }

  // Print history if requested and exit.
  if args.history {
    return print_history(&config, history_manager);
  }

  let mut file = match args.file.unwrap_or("-".to_string()).as_str() {
    "-" => File::from_stdin(),
    "@" => match clippy.as_mut().expect("Clipboard is not enabled.").get() {
      Ok(file) => file,
      Err(err) => panic!("{}", err),
    },
    file => File::from_path(&PathBuf::from(file)),
  };

  if let Some(filename) = args.filename {
    file.override_name(filename);
  }

  if let Some(mime) = args.mime {
    file.override_mime(mime);
  }

  templator.set_file(file.clone());

  let mut curlyfry = CurlyFry::new();
  curlyfry.set_url(templator.format(&config.uploader.request.url));
  curlyfry.set_method(config.uploader.request.method.clone());
  curlyfry.set_headers(config.uploader.request.headers.clone().into_iter().map(|(k, v)| (k, templator.format(&v))).collect());
  curlyfry.set_file(file.clone());
  let progress_notification = Notification::new()
  .appname("Uploadr")
  .summary("Uploading...")
  .timeout(0)
  .body(&format!("Uploading {}...", file.name))
  .show()
  .unwrap();
  curlyfry.execute(&config);
  
  templator.set_curlyfry(&curlyfry);

  if curlyfry.response.as_ref().unwrap().ok() {
    progress_notification.close();
    println!("Upload successful!");
    if config.data.notification.enabled {
      let url = templator.format(&config.uploader.response.url);
      let manage_url = config.uploader.response.manage_url.as_ref().map(|manage_url| templator.format(manage_url));
      let timeout = config.data.notification.timeout as i32 * 1000;
      // spawn a thread to display the notification
      std::thread::spawn(move || {
        let mut notification = Notification::new();
        notification.appname("Uploadr")
        .summary("Upload successful!")
        .body("Click to open in your browser.")
        .timeout(timeout)
        .action("open", "Open")
        .action("default", "Open");
        if manage_url.is_some() {
          notification.action("manage", "Manage Upload");
        }
        notification.show()
        .expect("Failed to show notification.")
        .wait_for_action(|action| {
          match action {
            "open" |
            "default" => {
              let _ = open(&url);
            },
            "manage" => {
              if let Some(manage_url) = manage_url{
                open(&manage_url).unwrap();
              }
            },
            _ => {},
          }
        });
      });
    }
    let url = templator.format(&config.uploader.response.url);
    let manage_url = config.uploader.response.manage_url.as_ref().map(|manage_url| templator.format(manage_url));
    let thumbnail_url = config.uploader.response.thumbnail_url.as_ref().map(|thumbnail_url| templator.format(thumbnail_url));
    if config.data.archive.enabled {
      history_manager.load();
      history_manager.add(&file, url.clone(), manage_url.clone(), thumbnail_url.clone());
      history_manager.save();
    }
    println!("URL: {}", url);
    if manage_url.is_some() {
      println!("Manage URL: {}", manage_url.unwrap());
    }
    if thumbnail_url.is_some() {
      println!("Thumbnail URL: {}", thumbnail_url.unwrap());
    }
    if !config.data.clipboard.read_only {
      clippy.expect("Clipboard is not enabled.").set(url);
      ExitCode::SUCCESS
    } else {
      println!("Clipboard is read-only, not copying URL to clipboard.");
      ExitCode::SUCCESS
    }
  } else {
    progress_notification.close();
    println!("Upload failed!");
    println!("Status Code: {}", curlyfry.response.as_ref().unwrap().status_code);
    println!("Response: {}", curlyfry.response.as_ref().unwrap().body);
    Notification::new()
    .appname("Uploadr")
    .summary("Upload failed!")
    .timeout(0)
    .body(&format!("Status Code: {}\nResponse: {}", curlyfry.response.as_ref().unwrap().status_code, curlyfry.response.as_ref().unwrap().body))
    .show()
    .unwrap();
    ExitCode::FAILURE
  }
}

fn print_history(config: &Config, mut history_manager: HistoryManager) -> ExitCode {
  if config.data.archive.enabled {
    if history_manager.load() {
      let history = history_manager.list();
      println!("File Name, URL, Manage URL, Thumbnail URL");
      println!("----------------");
      println!("{history}");
      ExitCode::SUCCESS
    } else {
      println!("No upload history found.");
      ExitCode::FAILURE
    }
  } else {
    println!("Upload history is disabled.");
    ExitCode::FAILURE
  }
}