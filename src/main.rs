mod utils;
use utils::config::Config;
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
  println!("{:?}", config);
}
