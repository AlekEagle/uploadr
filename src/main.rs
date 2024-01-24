mod utils;
use std::path::PathBuf;

use utils::config::Config;
use utils::history::HistoryManager;
use utils::templator::Templator;
use utils::curlyfries::CurlyFry;
use utils::file::File;
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
  #[clap(long)]
  pub history: bool,

  /// The file to upload.
  /// If not specified, the file will be read from stdin.
  /// If specified as "-", the file will be read from stdin.
  /// If specified as "@", the file will be read from the clipboard.
  pub file: Option<String>,
}

fn main() {
  let mut args: Args = Args::parse();
  let config = Config::new(args.config.take(), args.uploader.take());
  let mut templator = Templator::new(&config);
  let history_manager = HistoryManager::new(&config);
  let file = match args.file.unwrap_or("-".to_string()).as_str() {
    "-" => File::from_stdin(),
    "@" => File::new(&[], None), // File::from_clipboard();
    file => File::from_path(&PathBuf::from(file)),
  };
  templator.set_file(file.clone());

  if args.history {
    print_history(&config, history_manager);
  }

  let mut curlyfry = CurlyFry::new();
  curlyfry.set_url(templator.format(&config.uploader.request.url));
  curlyfry.set_method(config.uploader.request.method.clone());
  curlyfry.set_headers(config.uploader.request.headers.clone().into_iter().map(|(k, v)| (k, templator.format(&v))).collect());
  curlyfry.set_file(file);
  curlyfry.execute(&config);
  
  templator.set_curlyfry(&curlyfry);

  if curlyfry.response.as_ref().unwrap().status_code / 100 == 2 {
    println!("{}", templator.format("{ response:headers;Content-Type }"));
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
