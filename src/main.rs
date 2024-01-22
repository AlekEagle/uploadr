mod utils;
use utils::config::Config;
use utils::history::HistoryManager;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "Alek Evans", version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Args {
  /// The path to the config file to use.
  #[clap(short, long)]
  pub config_path: Option<String>,

  /// The uploader to use.
  #[clap(short, long)]
  pub uploader: Option<String>,

  /// Displays upload history.
  #[clap(long)]
  pub history: bool,
}

fn main() {
  let mut args: Args = Args::parse();
  let config = Config::new(args.config_path.take(), args.uploader.take());
  let history_manager = HistoryManager::new(&config);
  if args.history {
    print_history(&config, history_manager);
  }
}

fn print_history(config: &Config, mut history_manager: HistoryManager) -> () {
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
