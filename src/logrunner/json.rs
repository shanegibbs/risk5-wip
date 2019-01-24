use super::*;

mod lineiterator;

pub(crate) use lineiterator::LineIterator;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub(crate) enum LogLine {
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "insn")]
    Insn(JsonInsn),
    #[serde(rename = "state")]
    State(JsonState),
    #[serde(rename = "load")]
    Load(JsonMemory),
    #[serde(rename = "store")]
    Store(JsonMemory),
    #[serde(rename = "mem")]
    Memory(JsonMemory),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonLogTuple {
    pub line: usize,
    pub state: JsonState,
    pub insn: Option<JsonInsn>,
    pub store: Option<JsonMemory>,
    pub mems: Vec<JsonMemory>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonInsn {
    core: usize,
    pc: String,
    bits: String,
    desc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonState {
    id: usize,
    pc: String,
    prv: String,

    mstatus: String,
    mepc: String,
    mtval: String,
    mscratch: String,
    mtvec: String,
    mcause: String,
    minstret: String,
    mie: String,
    mip: String,
    medeleg: String,
    mideleg: String,
    mcounteren: String,
    scounteren: String,
    sepc: String,
    stval: String,
    sscratch: String,
    stvec: String,
    satp: String,
    scause: String,

    xregs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonMemory {
    #[serde(rename = "type")]
    kind: String,
    addr: String,
    value: String,
}

impl JsonLogTuple {
    pub fn to_logtuple(self) -> LogTuple {
        LogTuple {
            line: self.line,
            state: self.state.into(),
            insn: self.insn.map(|i| i.into()),
            store: self.store.map(|n| n.into()),
            mems: self.mems.into_iter().map(|n| n.into()).collect(),
        }
    }
}

impl Into<Insn> for JsonInsn {
    fn into(self) -> Insn {
        Insn {
            pc: string_to_u64(&self.pc),
            bits: string_to_u32(&self.bits),
            desc: self.desc,
        }
    }
}

impl Into<State> for JsonState {
    fn into(self) -> State {
        State {
            id: self.id,
            pc: string_to_u64(&self.pc),
            prv: string_to_u64(&self.prv),
            mstatus: string_to_u64(&self.mstatus),
            mscratch: string_to_u64(&self.mscratch),
            mtvec: string_to_u64(&self.mtvec),
            mcause: string_to_u64(&self.mcause),
            mepc: string_to_u64(&self.mepc),
            minstret: string_to_u64(&self.minstret),
            mie: string_to_u64(&self.mie),
            mip: string_to_u64(&self.mip),
            medeleg: string_to_u64(&self.medeleg),
            mideleg: string_to_u64(&self.mideleg),
            mcounteren: string_to_u64(&self.mcounteren),
            scounteren: string_to_u64(&self.scounteren),
            sepc: string_to_u64(&self.sepc),
            stval: string_to_u64(&self.stval),
            sscratch: string_to_u64(&self.sscratch),
            stvec: string_to_u64(&self.stvec),
            satp: string_to_u64(&self.satp),
            scause: string_to_u64(&self.scause),
            xregs: self.xregs.iter().map(|n| string_to_u64(n)).collect(),
        }
    }
}

impl Into<MemoryTrace> for JsonMemory {
    fn into(self) -> MemoryTrace {
        let JsonMemory { kind, addr, value } = self;
        MemoryTrace {
            kind: kind,
            addr: string_to_u64(&addr),
            value: string_to_u64(&value),
        }
    }
}

fn string_to_u64(s: &String) -> u64 {
    u64::from_str_radix(&s[2..], 16).expect("hex parse")
}

fn string_to_u32(s: &String) -> u32 {
    u32::from_str_radix(&s[2..], 16).expect("hex parse")
}
