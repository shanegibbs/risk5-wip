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

use std::io::{Cursor, Read};

pub(crate) struct LogLineReader<T> {
    buf: Cursor<Vec<u8>>,
    sz: usize,
    reader: T,
}

impl<T: Read> LogLineReader<T> {
    pub fn new(reader: T) -> Self {
        // let reader = io::BufReader::new(io::stdin());
        let buf = Cursor::new(vec![0; 0]);
        let mut lr = LogLineReader { buf, sz: 0, reader };
        // lr.skip_lines(100_000_000);
        lr
    }

    fn skip_lines(&mut self, n: usize) {
        for _ in 0..n {
            let mut buf = [0; 2];
            self.reader.read_exact(&mut buf).expect("read sz");
            let p1 = (buf[1] as usize) << 8;
            let sz = (buf[0] as usize) + p1;
            io::copy(&mut self.reader.by_ref().take(sz as u64), &mut io::sink())
                .expect("copy skip_lines");
        }
        self.sz = 0;
        self.buf = Cursor::new(vec![0; 0]);
    }

    fn fill_buffer(&mut self) -> bool {
        let mut buf = [0; 2];
        if let Err(e) = self.reader.read_exact(&mut buf) {
            warn!("failed to read: {}", e);
            return false;
        }
        let p1 = (buf[1] as usize) << 8;
        let sz = (buf[0] as usize) + p1;
        self.sz = sz;

        trace!("Reading {} bytes to buffer", sz);
        let mut buf = vec![0; sz];
        self.reader
            .read_exact(buf.as_mut_slice())
            .expect("read buf");
        self.buf = Cursor::new(buf);
        true
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
        if self.sz == self.buf.position() as usize {
            if !self.fill_buffer() {
                return None;
            }
        }

        let val = match bincode::deserialize_from(&mut self.buf) {
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

        trace!("Read: {:?}", val);
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
