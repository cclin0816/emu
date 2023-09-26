#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

#[cfg(feature = "F")]
mod fpu;

mod decode;
mod execute;
mod hart;
mod memory;
mod privilege;
mod uop;
mod utils;
mod xlen;

// use {hart::Hart, xlen::XlenT};

// pub fn tt() {
//     let mut hart = Hart::<u32, false>::new();
//     hart.run();
// }
