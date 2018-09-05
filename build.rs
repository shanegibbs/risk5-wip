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

    let mut output: Vec<String> = vec![];

    let opcode_f = File::open("opcodes").unwrap();
    let opcode_file = BufReader::new(&opcode_f);

    let opcode_re = Regex::new(r"(.+)=(.+)").unwrap();

    let mut first = true;
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

        let name_macro = format!("{}_insn", name);

        let mut args = vec![];
        let mut matches = vec![];

        for i in 1..parts.len() {
            let opcode_caps = opcode_re.captures(parts[i]);
            match opcode_caps {
                None => args.push(parts[i]),
                Some(_m) => matches.push(parts[i]),
            }

        }

        let mut cond = format!("}} else if");
        if first {
            cond = format!("if");
            first = false;
        }
        // output.push(format!("\n// {}", line));
        // output.push(format!("// args={:?}", args));

        let (mtch, mask) = build_match_mask(&matches, &mut output);
        // output.push(format!("// match=0x{:x}, mask=0x{:x}", mtch, mask));

        output.push(format!("{cond} insn & 0x{mask:x} == 0x{mtch:x} {{",
                            cond = cond,
                            mask = mask,
                            mtch = mtch));
        let mut vars = args.iter()
            .map(|a| format!("    let {a} = insn.{a}();", a = a))
            .collect::<Vec<_>>();
        output.append(&mut vars);

        let mut print_vars = args.iter()
            .map(|a| format!("{a}=0x{{{a}:x}}", a = a))
            .collect::<Vec<_>>()
            .join(" ");
        let mut print_vars_args = args.iter()
            .map(|a| format!("{a}={a}", a = a))
            .collect::<Vec<_>>()
            .join(",");
        output.push(format!("    info!(\"{} {}\", {});",
                            name,
                            print_vars,
                            print_vars_args));

        let fn_args = args.join(", ");
        let mut exec = format!("unimplemented_insn!(\"{}\");", name);
        if opcodes_rs.contains(&name_macro) {
            exec = format!("{}!({});", name_macro, fn_args);
        }

        output.push(format!("    {exec}", exec = exec));

    }
    output.push(format!("}} else {{"));
    output.push(format!("    unmatched_insn!()"));
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

    let existing_f = File::open("src/opcodes.rs").unwrap();
    let mut existing_file = BufReader::new(&existing_f);

    let mut contents = String::new();
    existing_file
        .read_to_string(&mut contents)
        .expect("something went wrong reading src/opcodes.rs");
    let contents = contents;

    let mut ifstmt: Vec<String> = vec![];
    let mut names: Vec<String> = vec![];

    let opcode_f = File::open("opcodes.txt").unwrap();
    let opcode_file = BufReader::new(&opcode_f);

    let mut first = true;
    for line in opcode_file.lines() {
        let line = line.unwrap();
        let mut split = line.split(" ");
        let _t = split.next().unwrap();
        let name_raw = split.next().unwrap();
        let name_var = name_raw.replace(".", "_");
        let name_const = name_var.to_uppercase();
        let mtch = split.next().unwrap();
        let mask = split.next().unwrap();
        let name_macro = format!("{}_inst", name_var);

        // output.push(format!("// {name_raw}\nlet {name_const}_MATCH: u32 = {mtch};\nlet {name_const}_MASK: u32 = {mask};\n",
        //                  name_raw = name_raw,
        //                  name_const = name_const,
        //                  mtch = mtch,
        //                  mask = mask));
        names.push(name_raw.to_owned());

        let mut exec = format!("unimplemented_inst!(\"{}\");", name_const);
        if contents.contains(&name_macro) {
            exec = format!("{}!();", name_macro);
        }

        let mut cond = format!("else if");
        if first {
            cond = format!("if");
            first = false;
        }

        ifstmt.push(format!("{cond} i & {mask} == {mtch} {{ info!(\"{name}\"); {exec} }}",
                            name = name_raw,
                            exec = exec,
                            cond = cond,
                            mtch = mtch,
                            mask = mask));
    }
    ifstmt.push(format!("else {{ unmatched_inst!(); }}"));
    // output.push("".to_owned());

    // output.push(format!("macro_rules! handle_inst_generated {{"));
    // output.push(format!("  () => ("));
    // let mut first = true;
    // for name in names {
    //     let name_const = name.replace(".", "_").to_uppercase();
    //     let name = name.replace(".", "_");
    //     let name_macro = format!("{}_inst", name);
    //     let mut exec = format!("unimplemented_inst!(\"{}\");", name_const);
    //     if contents.contains(&name_macro) {
    //         exec = format!("{}!();", name_macro);
    //     }

    //     let mut cond = format!("else if");
    //     if first {
    //         cond = format!("if");
    //         first = false;
    //     }

    //     output.push(format!("{cond} i & {name_const}_MASK == {name_const}_MATCH {{ {exec} }}",
    //                         cond = cond,
    //                         name_const = name_const,
    //                         exec = exec));
    // }
    // output.push(format!("else {{ unmatched_inst!(); }}"));

    let out_dir = env::var("OUT_DIR").unwrap();

    let ifstmt = ifstmt.join("\n");
    let dest_path = Path::new(&out_dir).join("opcodes_ifstmt.rs");
    let f = File::create(&dest_path).unwrap();
    let mut f = BufWriter::new(&f);
    f.write_all(ifstmt.as_bytes())
        .expect("Unable to write ifstmt");
}
