// common decoding functions

use crate::{hart::HartIsa, utils::Maybe, xlen::XlenT};

pub const ZERO: u8 = 0;
pub const RA: u8 = 1;
pub const SP: u8 = 2;

/// sign-extend any length imm to i32
pub fn sext(imm: u32, sign_bit: u32) -> i32 {
    let len = 31 - sign_bit;
    let imm = (imm << len) as i32;
    imm >> len
}

macro_rules! if_ge_rv64 {
    ($e_true:expr, $e_false:expr) => {{
        #[cfg(feature = "RV64")]
        if Xlen::XLEN >= 64 {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = "RV64"))]
        {
            $e_false
        }
    }};
    ($e:expr) => {
        if_ge_rv64!(Ok($e), Err(()))
    };
}

macro_rules! if_rv128 {
    ($e_true:expr, $e_false:expr) => {{
        #[cfg(feature = "RV128")]
        if Xlen::XLEN == 128 {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = "RV128"))]
        {
            $e_false
        }
    }};
    ($e:expr) => {
        if_rv128!(Ok($e), Err(()))
    };
}

/// # Arguments
///
/// * `id` - extension id, e.g. `A`
/// * `lit` - extension literal, e.g. `"A"`
/// * `self` - `self` (`HartIsa`) reference
/// * `e_true` - expression to be evaluated if `self.id` is enabled
/// * `e_false` - expression to be evaluated if `self.id` is disabled
///
/// if e_false is not provided, it will be `Err(())`
/// and `e_true` will be `Ok(e_true)`
///
/// # Note
///
/// `lit` is required since rust can't stringify ident,
/// see [this](https://www.reddit.com/r/rust/comments/snmses/how_to_use_stringify_inside_cfgfeature/)
///
macro_rules! if_ext {
    ($id:ident, $lit:literal, $self:ident, $e_true:expr, $e_false:expr) => {{
        #[cfg(feature = $lit)]
        if $self.$id {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = $lit))]
        {
            $e_false
        }
    }};
    ($id:ident, $lit:literal, $self:ident, $e:expr) => {
        if_ext!($id, $lit, $self, Ok($e), Err(()))
    };
}

macro_rules! if_ext_a {
    ($self:ident, $($e:expr),*) => {
        if_ext!(A, "A", $self, $($e),*)
    };
}

macro_rules! if_ext_c {
    ($self:ident, $($e:expr),*) => {
        if_ext!(C, "C", $self, $($e),*)
    };
}

macro_rules! if_ext_d {
    ($self:ident, $($e:expr),*) => {
        if_ext!(D, "D", $self, $($e),*)
    };
}

macro_rules! if_ext_f {
    ($self:ident, $($e:expr),*) => {
        if_ext!(F, "F", $self, $($e),*)
    };
}

macro_rules! if_ext_m {
    ($self:ident, $($e:expr),*) => {
        if_ext!(M, "M", $self, $($e),*)
    };
}

macro_rules! if_ext_zicsr {
    ($self:ident, $($e:expr),*) => {
        if_ext!(Zicsr, "Zicsr", $self, $($e),*)
    };
}

macro_rules! if_ext_zifencei {
    ($self:ident, $($e:expr),*) => {
        if_ext!(Zifencei, "Zifencei", $self, $($e),*)
    };
}

/// `result` = `val`\[`high`:`low`\]
pub fn select_bits<T>(val: T, high: u8, low: u8) -> T
where
    T: Copy
        + From<u8>
        + std::ops::Shr<Output = T>
        + std::ops::Shl<Output = T>
        + std::ops::BitAnd<Output = T>
        + std::ops::Sub<Output = T>,
{
    let one = T::from(1);
    let len = T::from(high - low + 1);
    let mask = (one << len) - one;
    (val >> T::from(low)) & mask
}

/// test if `bit` is set in `val`
pub fn test_bit<T>(val: T, bit: u8) -> bool
where
    T: Copy + From<u8> + std::ops::Shl<Output = T> + std::ops::BitAnd<Output = T> + std::cmp::Eq,
{
    let mask = T::from(1) << T::from(bit);
    val & mask == mask
}

/// group_x = `val`\[`high_x`:`low_x`\]\
/// result = \[`...` : `group_1` : `group_0` : `shift`\]
///
/// # Arguments
///
/// * `val` - value to be shuffled
/// * `shift` - shift amount
/// * `(high, low)+` - bit groups to be selected
///
macro_rules! shuffle_bits {
    ($val:expr, $shift:literal, $($high:literal, $low:literal),*) => {
        {
            let val = $val as u32;
            let mut res = 0;
            let mut _pos = $shift;
            $(
                res |= select_bits(val, $high, $low) << _pos;
                _pos += $high - $low + 1;
            )*
            res
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(select_bits(0b0011_1000, 6, 2), 0b01110);
        assert_eq!(select_bits(0b0011_1000, 5, 5), 0b1);
        assert!(!test_bit(0b0011_1000, 6));
        assert!(test_bit(0b0011_1000, 5));
        assert_eq!(shuffle_bits!(0b1010_0101, 2, 6, 4, 2, 2), 0b10_1000);
        assert_eq!(
            shuffle_bits!(0b0011_1100, 3, 4, 4, 7, 5, 3, 1),
            0b11_0001_1000
        );
    }
}
