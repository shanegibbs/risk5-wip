use super::*;
use std::{fmt, io};

pub(crate) struct BincodeReader {
    reader: BufReader<io::Stdin>,
}

impl BincodeReader {
    pub fn new() -> Self {
        let reader = BufReader::new(io::stdin());
        BincodeReader { reader }
    }
}

impl Iterator for BincodeReader {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        use bincode;

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
