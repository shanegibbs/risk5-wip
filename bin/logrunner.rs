use risk5;

fn main() -> Result<(), Box<std::error::Error>> {
    risk5::logrunner::run().map_err(|e| e.into())
}
