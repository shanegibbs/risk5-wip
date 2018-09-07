use std::path::PathBuf;

use elf;

pub fn read_program_segments() -> (u64, Vec<(u64, u64, u64)>) {
    let mut r = vec![];
    let path = PathBuf::from("../bins/bbl/bbl");
    let file = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error: {:?}", e),
    };

    let _text_scn = match file.get_section(".text") {
        Some(s) => s,
        None => panic!("Failed to look up .text section"),
    };

    debug!("ep 0x{:x}", file.ehdr.entry);

    for phdr in &file.phdrs {
        if phdr.progtype == elf::types::PT_LOAD && phdr.memsz > 0 {
            debug!("offset in file 0x{:x}", phdr.offset);
            debug!("mem offset 0x{:x}", phdr.vaddr);
            debug!("filesz 0x{:x}", phdr.filesz);
            r.push((phdr.offset, phdr.vaddr, phdr.filesz));
        }
    }
    return (file.ehdr.entry, r);
}
