#[macro_use]
extern crate log;
use pretty_env_logger;
use risk5;

fn main() {
    pretty_env_logger::init();
    match risk5::logrunner::bincodereader() {
        Err(e) => error!("{}", e),
        Ok(()) => (),
    }
}
