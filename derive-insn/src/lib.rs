extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;

use proc_macro::TokenStream;
use proc_macro::TokenTree;
use proc_macro2::*;

#[derive(Debug)]
struct InsnAttribute {
    kind: String,
    mask: u32,
    mtch: u32,
}

macro_rules! assert_token {
    ($i:expr, $t:path, $value:expr) => (match $i {
        $t(ref i) => assert_eq!(i.to_string(), $value),
        ref n => panic!("Expecting token {} to be '{}'. Was {:?}", $i, $value, n),
    })
}

macro_rules! get_string_token {
    ($v:expr) => (match $v {
        TokenTree::Ident(ref i) => i.to_string(),
        _ => panic!("Expecting String. Got {:?}", $v),
    })
}

macro_rules! get_hex_token {
    ($v:expr) => (u32::from_str_radix(&format!("{}", $v)[2..], 16).expect("hex string"))
}

fn get_token_stream(n: &TokenTree) -> Vec<TokenTree> {
    match n {
        TokenTree::Group(ref g) => g.stream().into_iter().collect::<Vec<_>>(),
        x => panic!(format!("Unexpected token: {}", x)),
    }
}

fn parse_insn_attribute(n: &TokenTree) -> InsnAttribute {
    let stream = get_token_stream(n);
    assert_token!(stream[0], TokenTree::Ident, "insn");

    let stream = get_token_stream(&stream[1]);
    assert_token!(stream[0], TokenTree::Ident, "kind");
    assert_token!(stream[1], TokenTree::Punct, "=");
    assert_token!(stream[3], TokenTree::Punct, ",");
    assert_token!(stream[4], TokenTree::Ident, "mask");
    assert_token!(stream[5], TokenTree::Punct, "=");
    assert_token!(stream[7], TokenTree::Punct, ",");
    assert_token!(stream[8], TokenTree::Ident, "match");
    assert_token!(stream[9], TokenTree::Punct, "=");

    let insn_kind = get_string_token!(stream[2]);
    let mask: u32 = get_hex_token!(stream[6]);
    let mtch: u32 = get_hex_token!(stream[10]);

    InsnAttribute {
        kind: insn_kind,
        mask: mask,
        mtch: mtch,
    }
}

#[proc_macro_attribute]
pub fn insns(attr: TokenStream, func: TokenStream) -> TokenStream {
    let _attr_vec: Vec<_> = attr.into_iter().collect();
    let func_vec: Vec<_> = func.into_iter().collect();

    let a = match &func_vec[3] {
        TokenTree::Group(ref a) => a.stream().into_iter().collect::<Vec<_>>(),
        ref n => panic!("Failed to get group. Got: {:?}", n),
    };
    let _b = match &a[1] {
        TokenTree::Group(ref b) => b.stream().into_iter().collect::<Vec<_>>(),
        ref n => panic!("Failed to get attr group. Got: {:?}", n),
    };

    // panic!(format!("{:?}", b));

    let mut mm = vec![];

    for (i, n) in a.iter().enumerate() {
        if let TokenTree::Punct(ch) = n {
            if ch.to_string() == format!("#") {
                mm.push(parse_insn_attribute(&a[i + 1]));
            }
        }
    }

    // panic!(format!("{:?}", mm));
    debug!("amc {:?}", mm);
    println!("amc {:?}", mm);
    func_vec.into_iter().collect()
}

#[proc_macro_attribute]
pub fn insn(attr: TokenStream, func: TokenStream) -> TokenStream {

    let attr_vec: Vec<_> = attr.into_iter().collect();
    let func_vec: Vec<_> = func.into_iter().collect();

    // panic!(format!("{:?}", attr_vec));
    // panic!(format!("{:?}", func_vec));
    // for (i, n) in func_vec.iter().enumerate() {
    //     println!("{}: {:?}", i, n);
    // }

    let insn_name = match func_vec[2] {
        TokenTree::Ident(ref i) => format!("{}", i),
        _ => panic!("No insn name"),
    };

    // vec of (arg_name, arg_type), e.g. ("rd", "u32")
    let args = {
        let fn_args: Vec<_> = match func_vec[8] {
            TokenTree::Group(ref g) => g.stream().into_iter().collect(),
            ref g => panic!("Expected group. Got {:?}", g),
        };

        // for (i, n) in fn_args.iter().enumerate() {
        //     println!("{}: {:?}", i, n);
        // }

        let arg_count = (fn_args.len() - 8) / 4;

        let mut args = vec![];
        for i in 0..arg_count {
            let arg_name = get_string_token!(fn_args[9 + i * 4]);
            let arg_type = get_string_token!(fn_args[11 + i * 4]);
            args.push((arg_name, arg_type));
        }
        args
    };

    // #[insn(kind=I,mask=0x110,match=0x100)]
    // 0    kind
    // 1    =
    // 2  I
    // 3    ,
    // 4    mask
    // 5    =
    // 6  0x100
    // 7    ,
    // 8    match
    // 9    =
    // 10 0x100

    assert_token!(attr_vec[0], TokenTree::Ident, "kind");
    assert_token!(attr_vec[1], TokenTree::Punct, "=");
    assert_token!(attr_vec[3], TokenTree::Punct, ",");
    assert_token!(attr_vec[4], TokenTree::Ident, "mask");
    assert_token!(attr_vec[5], TokenTree::Punct, "=");
    assert_token!(attr_vec[7], TokenTree::Punct, ",");
    assert_token!(attr_vec[8], TokenTree::Ident, "match");
    assert_token!(attr_vec[9], TokenTree::Punct, "=");

    let insn_kind = get_string_token!(attr_vec[2]);
    let mask: u32 = get_hex_token!(attr_vec[6]);
    let mtch: u32 = get_hex_token!(attr_vec[10]);

    let main_fn_name = Ident::new(&insn_name, Span::call_site());
    let desc_fn_name = Ident::new(&format!("{}_desc", insn_name), Span::call_site());
    let insn_fn_name = Ident::new(&format!("{}_exec", insn_name), Span::call_site());

    let debug_format_args = args.iter()
        .map(|a| format!("{}=0x{{:x}}", a.0))
        .collect::<Vec<_>>()
        .join(" ");
    let debug_format_string = format!("0x{{:x}} {} {}", insn_name, debug_format_args);

    let insn_desc_args = args.iter()
        .map(|a| {
                 format!("{}:{:?}:{}",
                         a.0,
                         insn_field_name_map(&insn_kind, &a.0),
                         a.1)
             })
        .collect::<Vec<_>>()
        .join(",");

    let insn_desc = format!("{} kind={}, mask=0x{:x}, match=0x{:x}, args={}",
                            insn_name,
                            insn_kind,
                            mask,
                            mtch,
                            insn_desc_args);

    let declare_args = args.iter()
        .map(|a| {
            let n = Ident::new(&a.0, Span::call_site());
            let (fn_name, cast_as) = insn_field_name_map(&insn_kind, &a.0);
            let fn_name = Ident::new(fn_name, Span::call_site());
            let cast_as = Ident::new(cast_as, Span::call_site());
            quote! {
            let #n = i.#fn_name() as #cast_as;
        }
        })
        .collect::<Vec<_>>();

    let arg_names = args.iter()
        .map(|a| Ident::new(&a.0, Span::call_site()))
        .collect::<Vec<_>>();
    let arg_names = &arg_names;

    let code = quote! {

        pub fn #desc_fn_name() -> &'static str {
            #insn_desc
        }

        #[inline(always)]
        pub fn #insn_fn_name<M: Memory>(p: &mut Processor<M>, i: u32) {
            #(#declare_args)*
            debug!(#debug_format_string, p.pc(), #(#arg_names),*);
            #main_fn_name(p, #(#arg_names),*)
        }

        #[inline(always)]
    };

    let code: TokenStream = code.into();
    // code.into().append(func_vec)
    code.into_iter().chain(func_vec).collect()
}

// return (fn_name, cast to type)
fn insn_field_name_map(kind: &str, name: &str) -> (&'static str, &'static str) {
    match (kind, name) {
        ("I", "rd") => ("rd", "usize"),
        ("I", "rs") => ("rs1", "usize"),
        ("I", "imm") => ("imm12", "u32"),
        ("I", "csr") => ("imm12", "usize"),
        ("J", "rd") => ("rd", "usize"),
        ("J", "imm") => ("imm20", "u32"),
        ("U", "rd") => ("rd", "usize"),
        ("U", "imm") => ("imm20", "u32"),
        ("B", "rs1") => ("rs1", "usize"),
        ("B", "rs2") => ("rs2", "usize"),
        ("B", "lo") => ("bimm12lo", "u32"),
        ("B", "high") => ("bimm12hi", "u32"),
        _ => {
            panic!(format!("Unmatched insn field name '{}' for insn type '{}'",
                           name,
                           kind))
        }
    }

}
