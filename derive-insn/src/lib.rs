extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::*;
use proc_macro::TokenTree;

// #[proc_macro_derive(riscv, attributes(opcode))]
#[proc_macro_attribute]
pub fn insn(attr: TokenStream, func: TokenStream) -> TokenStream {

    let attr_vec: Vec<_> = attr.into_iter().collect();
    let func_vec: Vec<_> = func.into_iter().collect();

    // panic!(format!("{:?}", attr_vec));
    // panic!(format!("{:?}", func_vec));

    let insn_name = match func_vec[2] {
        TokenTree::Ident(ref i) => format!("{}", i),
        _ => panic!("No insn name"),
    };

    {
        let fn_args = match func_vec[3] {
            TokenTree::Group(ref g) => g,
            ref g => panic!("Expected group. Got {:?}", g),
        };
    }


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

    macro_rules! assert_token {
        ($i:expr, $t:path, $value:expr) => (match attr_vec[$i] {
            $t(ref i) => assert_eq!(i.to_string(), $value),
            ref n => panic!("Expecting token {} to be '{}'. Was {:?}", $i, $value, n),
        })
    }

    macro_rules! get_string_token {
        ($i:expr) => (match attr_vec[$i] {
            TokenTree::Ident(ref i) => i.to_string(),
            ref n => panic!("Expecting String at {}. Got {:?}", $i, n),
        })
    }

    macro_rules! get_hex_token {
        ($i:expr) => (u32::from_str_radix(&format!("{}", attr_vec[$i]), 16).expect("hex string"))
    }

    assert_token!(0, TokenTree::Ident, "kind");
    assert_token!(1, TokenTree::Punct, "=");
    assert_token!(3, TokenTree::Punct, ",");
    assert_token!(4, TokenTree::Ident, "mask");
    assert_token!(5, TokenTree::Punct, "=");
    assert_token!(7, TokenTree::Punct, ",");
    assert_token!(8, TokenTree::Ident, "match");
    assert_token!(9, TokenTree::Punct, "=");

    let insn_kind = get_string_token!(2);
    let mask: u32 = get_hex_token!(6);
    let mtch: u32 = get_hex_token!(10);

    // panic!(format!("{:?}", insn_name));

    let main_fn_name = Ident::new(&insn_name, Span::call_site());
    let desc_fn_name = Ident::new(&format!("{}_desc", insn_name), Span::call_site());
    let insn_fn_name = Ident::new(&format!("{}_exec", insn_name), Span::call_site());

    let insn_desc = format!("{} kind={}, mask=0x{:x}, match=0x{:x}",
                            insn_name,
                            insn_kind,
                            mask,
                            mtch);

    let code = quote! {

        pub fn #desc_fn_name() -> &'static str {
            #insn_desc
        }

        pub fn #insn_fn_name(p: &mut Processor, i: u32) {
            #main_fn_name(p, 1, 2, 3)
        }

        #[inline(always)]
    };

    let code: TokenStream = code.into();
    // code.into().append(func_vec)
    code.into_iter().chain(func_vec).collect()
}

/*
fn impl_riscv(ast: &syn::DeriveInput) -> quote::Tokens {
    // panic!(format!("{:?}", ast));
    // for a in &ast.attrs {
    //     panic!(format!("{:?}", a));
    // }
    let name = &ast.ident;
    // Check if derive(HelloWorld) was specified for a struct
    if let syn::Body::Struct(_) = ast.body {
        // Yes, this is a struct
        quote! {
            fn print_abc(i: u32) {
                println!("insn {}", i);
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        panic!("#[derive(HelloWorld)] is only defined for structs, not for enums!");
    }
}
*/
