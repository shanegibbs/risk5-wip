use pretty_env_logger;
use risk5;

fn main() -> Result<(), Box<std::error::Error>> {
    pretty_env_logger::init();
    risk5::logrunner::run().map_err(|e| e.into())
}
