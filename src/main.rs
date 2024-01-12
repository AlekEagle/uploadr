mod utils;
use utils::config::Config;
use clap::Parser;

fn main() {
  let mut args: Args = Args::parse();
  println!("Config path: {}", args.config_path.get_or_insert(Config::get_default_config_path()));
  println!("Default uploader: {}", args.uploader.unwrap());
  let config: Config = Config::new(None);
  println!("Config path: {}", config.config_path);
  println!("Default uploader: {}", config.config.defaultUploader);
  println!("Notification enabled: {}", config.config.notification.enabled);
}
