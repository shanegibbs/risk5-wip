use super::{json, LogLine, LogTuple};
use bincode;
use std::io;

pub fn bincodereader() -> Result<(), io::Error> {
    // let i = LogLine::Mark;
    // let n = bincode::serialize(&i).unwrap();
    // info!("{:?}", n);

    // let i = LogLine::Memory(super::MemoryTrace {
    //     kind: super::MemoryTraceKind::Uint16,
    //     addr: 3,
    //     value: 5,
    // });
    // let n = bincode::serialize(&i).unwrap();
    // info!("{:?}", n);

    // let n = vec![4, 0, 0, 0, 0, 0, 0, 0, 0];
    // let i: LogLine = bincode::deserialize(&n).expect("deser");
    // info!("{:?}", i);

    for line in LogLineReader::new(io::BufReader::new(io::stdin())) {
        info!("{:?}", line);
    }
    Ok(())
}

pub fn convert() -> Result<(), io::Error> {
    let mut out = io::BufWriter::new(io::stdout());

    for line in json::TupleIterator::new(json::LineIterator::new()) {
        trace!("{:?}", line);
        let bin = line.to_logtuple();
        bincode::serialize_into(&mut out, &bin).map_err(|e| match *e {
            bincode::ErrorKind::Io(e) => e,
            e => io::Error::new(io::ErrorKind::Other, format!("{}", e)),
        })?
    }
    Ok(())
}

pub(crate) struct LogLineReader<T> {
    reader: T,
}

impl<T> LogLineReader<T> {
    pub fn new(t: T) -> Self {
        // let reader = io::BufReader::new(io::stdin());
        LogLineReader { reader: t }
    }
}

impl<T: io::Read> LogLineReader<T> {
    pub fn to_tuple(self) -> LineToTupleIterator<LogLineReader<T>> {
        LineToTupleIterator::new(self)
    }
}

impl<T: io::Read> Iterator for LogLineReader<T> {
    type Item = LogLine;

    fn next(&mut self) -> Option<LogLine> {
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
                error!("Failed to read log line: {}", e);
                panic!("Failed to read log line");
            }
        };

        Some(val)
    }
}

pub(crate) struct TupleReader {
    reader: io::BufReader<io::Stdin>,
}

impl TupleReader {
    pub fn new() -> Self {
        let reader = io::BufReader::new(io::stdin());
        TupleReader { reader }
    }
}

impl Iterator for TupleReader {
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

pub(crate) struct LineToTupleIterator<I> {
    line_it: I,
}

impl<I> LineToTupleIterator<I> {
    pub fn new(mut it: I) -> Self
    where
        I: Iterator<Item = LogLine>,
    {
        loop {
            match it.next() {
                None => break,
                Some(LogLine::Mark) => break,
                _ => (),
            }
        }
        loop {
            match it.next() {
                None => break,
                Some(LogLine::Mark) => break,
                _ => (),
            }
        }

        LineToTupleIterator { line_it: it }
    }
}

impl<I> Iterator for LineToTupleIterator<I>
where
    I: Iterator<Item = LogLine>,
{
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        let mut insn = None;
        let mut state = None;
        let mut store = None;
        let mut mems = vec![];

        loop {
            let next = match self.line_it.next() {
                Some(ll) => ll,
                None => return None,
            };

            match next {
                LogLine::Mark => break,
                LogLine::Insn(n) => insn = Some(n),
                LogLine::State(n) => state = Some(n),
                LogLine::Memory(n) => mems.push(n),
                LogLine::Load(_) => (),
                LogLine::Store(n) => store = Some(n),
            }
        }

        let state = state.expect("state");

        Some(LogTuple {
            line: 0,
            state,
            insn,
            store,
            mems,
        })
    }
}
