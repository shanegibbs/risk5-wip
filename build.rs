extern crate regex;

use regex::Regex;

use std::env;
use std::io::*;
use std::fs::*;
use std::path::*;

fn build_match_mask(input: &[&str], output: &mut Vec<String>) -> (u32, u32) {
    let mut final_mask = 0;
    let mut final_value = 0;
    let range_re = Regex::new(r"([0-9]+)\.\.([0-9]+)=(.+)").unwrap();

    for inp in input {
        let range_caps = range_re.captures(inp);
        if let Some(caps) = range_caps {
            let end: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
            let start: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
            let value_str = caps.get(3).unwrap().as_str();
            let mut value: u32 = 0;
            if value_str.starts_with("0x") {
                value = u32::from_str_radix(&value_str[2..], 16).unwrap() << start;
            } else {
                value = value_str.parse::<u32>().unwrap() << start;
            }
            final_value |= value;

            let mut mask: u32 = 0;
            for i in start..end + 1 {
                mask |= 1 << i;
            }
            final_mask |= mask;

            // output.push(format!("// mm {}-{}={}, value={:b}, mask={:b}",
            //                     start,
            //                     end,
            //                     value_str,
            //                     value,
            //                     mask));
        }
    }

    // output.push(format!("// value=0x{value:x} / {value:b} mask = {mask:b}",
    //                     value = final_value,
    //                     mask = final_mask));

    (final_value, final_mask)
}

fn new_style() {
    let opcodes_rs = {
        let existing_f = File::open("src/opcodes.rs").unwrap();
        let mut existing_file = BufReader::new(&existing_f);
        let mut contents = String::new();
        existing_file
            .read_to_string(&mut contents)
            .expect("something went wrong reading src/opcodes.rs");
        contents
    };

    let mut output: Vec<String> = vec![format!("{{")];

    let opcode_f = File::open("opcodes").unwrap();
    let opcode_file = BufReader::new(&opcode_f);

    let opcode_re = Regex::new(r"(.+)=(.+)").unwrap();

    for line in opcode_file.lines() {
        let line = line.unwrap();
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0];
        if name.starts_with("#") {
            continue;
        }

        let name_fn = format!("{}", name);

        let mut args = vec![];
        let mut matches = vec![];

        for i in 1..parts.len() {
            let opcode_caps = opcode_re.captures(parts[i]);
            match opcode_caps {
                None => args.push(parts[i]),
                Some(_m) => matches.push(parts[i]),
            }

        }

        let (mtch, mask) = build_match_mask(&matches, &mut output);

        let mut exec = format!("unimplemented");
        if opcodes_rs.contains(&format!("fn {}(", name_fn)) {
            exec = format!("{}", name_fn);
        }

        output.push(format!("insns.push((0x{mask:x}, 0x{mtch:x}, {exec}, \"{name}\")); // info!(\"Adding insn\");",
                            exec = exec,
                            name = name,
                            mask = mask,
                            mtch = mtch));

    }
    output.push(format!("}}"));

    let out_dir = env::var("OUT_DIR").unwrap();
    let ifstmt = output.join("\n");
    let dest_path = Path::new(&out_dir).join("insns.rs");
    let f = File::create(&dest_path).unwrap();
    let mut f = BufWriter::new(&f);
    f.write_all(ifstmt.as_bytes())
        .expect("Unable to write isns.rs");
}

fn main() {
    new_style();
}
