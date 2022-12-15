use log::Level::Info;
use log::log_enabled;
use env_logger;

fn main() {
  env_logger::init();
  if log_enabled!(Info) {
    println!("Info enabled");
  } else {
    println!("Info disabled");
  }
}