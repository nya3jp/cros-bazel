use env_logger;
use log::log_enabled;
use log::Level::Info;

fn main() {
    env_logger::init();
    if log_enabled!(Info) {
        println!("Info enabled");
    } else {
        println!("Info disabled");
    }

    println!(
        "RUNFILES DIR IS {:?}",
        runfiles::find_runfiles_dir().unwrap()
    )
}
