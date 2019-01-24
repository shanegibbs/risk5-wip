struct Field<T> {
    internal: T,
    offset: u8,
    width: u8,
}

impl<T> Field<T> {
    fn new(n: T, offset: u8, width: u8) -> Self {
        Field {
            internal: n,
            offset: offset,
            width: width,
        }
    }
}

struct SignExtended<T> {
    field: Field<T>,
}

impl From<SignExtended<u32>> for i32 {
    #[inline(always)]
    fn from(n: SignExtended<u32>) -> i32 {
        let extend = 32 - n.field.width;
        let a: u32 = n.field.into();
        (a as i32) << extend >> extend
    }
}

impl From<Field<u32>> for i32 {
    #[inline(always)]
    fn from(n: Field<u32>) -> i32 {
        let extend = 32 - n.width;
        let a: u32 = n.into();
        (a as i32) << extend >> extend
    }
}

impl Field<u32> {
    #[inline(always)]
    fn sign_extend(self) -> SignExtended<u32> {
        SignExtended { field: self }
    }
}

impl From<u32> for Field<u32> {
    #[inline(always)]
    fn from(n: u32) -> Field<u32> {
        Field {
            internal: n,
            offset: 0,
            width: 32,
        }
    }
}

impl From<Field<u32>> for u32 {
    #[inline(always)]
    fn from(n: Field<u32>) -> u32 {
        n.internal >> n.offset & (1 << n.width) - 1
    }
}

#[inline(never)]
fn print_u32<T: Into<u32>>(t: T) {
    let n: u32 = t.into();
    println!("{}", n);
}

#[inline(never)]
fn print_i32<T: Into<i32>>(t: T) {
    let n: i32 = t.into();
    println!("{}", n);
}

#[inline(never)]
fn dowork<T: Into<i32>>(t: T) {
    print_i32(t.into())
}

use std::env::vars;

fn main() {
    let v = vars().count();

    let n = Field::new(v as u32, 1, 2);
    print_u32(n);

    // let n = Field::new(v as u32, 1, 2).sign_extend();
    let n = Field::new(v as u32, 1, 2);
    // println!("{:?}", n);
    // print_i32(n);
    dowork(n);
}
