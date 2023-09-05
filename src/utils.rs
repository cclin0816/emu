pub type Maybe<T> = Result<T, ()>;

/// select_bits(0b0011_1000, 6, 2) -> 0b01110
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

pub fn select_bit<T>(val: T, bit: u8) -> bool
where
    T: Copy + From<u8> + std::ops::Shl<Output = T> + std::ops::BitAnd<Output = T> + std::cmp::Eq,
{
    let mask = T::from(1) << T::from(bit);
    val & mask == mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_bits() {
        assert_eq!(select_bits(0b0011_1000, 6, 2), 0b01110);
        assert_eq!(select_bits(0b0011_1000, 5, 5), 0b1);
    }

    #[test]
    fn test_select_bit() {
        assert!(!select_bit(0b0011_1000, 6));
        assert!(select_bit(0b0011_1000, 5));
    }
}
