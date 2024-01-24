mod utils;
use std::path::PathBuf;

use utils::config::Config;
use utils::history::HistoryManager;
use utils::templator::Templator;
use utils::curlyfries::CurlyFry;
use utils::file::File;
use utils::clippy::Clippy;
use clap::Parser;

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

fn main() {
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
    print_history(&config, history_manager);
    return;
  }

  let mut file = match args.file.unwrap_or("-".to_string()).as_str() {
    "-" => File::from_stdin(),
    "@" => match clippy.as_mut().expect("Clipboard is not enabled.").get() {
      Ok(file) => file,
      Err(err) => panic!("{}", err),
    },
    file => File::from_path(&PathBuf::from(file)),
  };

  if args.filename.is_some() {
    file.override_name(args.filename.unwrap());
  }

  if args.mime.is_some() {
    file.override_mime(args.mime.unwrap());
  }

  templator.set_file(file.clone());

  let mut curlyfry = CurlyFry::new();
  curlyfry.set_url(templator.format(&config.uploader.request.url));
  curlyfry.set_method(config.uploader.request.method.clone());
  curlyfry.set_headers(config.uploader.request.headers.clone().into_iter().map(|(k, v)| (k, templator.format(&v))).collect());
  curlyfry.set_file(file.clone());
  curlyfry.execute(&config);
  
  templator.set_curlyfry(&curlyfry);

  if curlyfry.response.as_ref().unwrap().status_code / 100 == 2 {
    println!("Upload successful!");
    let url = templator.format(&config.uploader.response.url);
    let manage_url = match config.uploader.response.manage_url {
      Some(ref manage_url) => Some(templator.format(manage_url)),
      None => None,
    };
    let thumbnail_url = match config.uploader.response.thumbnail_url {
      Some(ref thumbnail_url) => Some(templator.format(thumbnail_url)),
      None => None,
    };
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
    if config.data.clipboard.enabled {
      clippy.expect("Clipboard is not enabled.").set(url);
    }
  } else {
    println!("Upload failed!");
  }
}

fn print_history(config: &Config, mut history_manager: HistoryManager) {
  if config.data.archive.enabled {
    if history_manager.load() {
      let history = history_manager.list();
      println!("File Name, URL, Manage URL, Thumbnail URL");
      println!("----------------");
      println!("{history}");
    } else {
      println!("No upload history found.");
    }
  } else {
    println!("Upload history is disabled.");
  }
}
