#[macro_use]
mod common;
#[cfg(feature = "C")]
mod dec16;
mod dec32;

// reserve other than hint raise illegal instruction exception