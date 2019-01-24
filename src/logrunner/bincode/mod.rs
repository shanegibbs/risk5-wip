use super::{json, logtupleiterator, LogTuple};
use bincode;
use std::io;

pub fn convert() -> Result<(), io::Error> {
    let mut out = io::BufWriter::new(io::stdout());

    let it = json::LineIterator::new();
    for line in logtupleiterator::LogTupleIterator::new(it) {
        trace!("{:?}", line);
        let bin = line.to_logtuple();
        bincode::serialize_into(&mut out, &bin).map_err(|e| match *e {
            bincode::ErrorKind::Io(e) => e,
            e => io::Error::new(io::ErrorKind::Other, format!("{}", e)),
        })?
    }
    Ok(())
}

pub(crate) struct BincodeReader {
    reader: io::BufReader<io::Stdin>,
}

impl BincodeReader {
    pub fn new() -> Self {
        let reader = io::BufReader::new(io::stdin());
        BincodeReader { reader }
    }
}

impl Iterator for BincodeReader {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        let val = match bincode::deserialize_from(&mut self.reader) {
            Ok(n) => n,
            Err(e) => {
                match *e {
                    bincode::ErrorKind::Io(ref e) => {
                        if e.kind() == io::ErrorKind::UnexpectedEof {
                            return None;
                        }
                    }
                    _ => (),
                }
                error!("Failed to read log tuple: {}", e);
                panic!("Failed to read log tuple");
            }
        };

        Some(val)
    }
}
