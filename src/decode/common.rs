// common decoding functions

use crate::{hart::HartIsa, utils::Maybe, xlen::XlenT};

impl<Xlen: XlenT, const EMB: bool> HartIsa<Xlen, EMB> {
    pub fn check_gp_reg(reg: u8) -> Maybe<()> {
        #[cfg(feature = "E")]
        if EMB && reg >= 16 {
            return Err(());
        }
        Ok(())
    }
}

macro_rules! check_gp_regs {
    ($($reg:ident),*) => {
        $(
            HartIsa::<Xlen, EMB>::check_gp_reg($reg)?;
        )*
    };
}

macro_rules! en_if_ge_rv64 {
    ($e:expr) => {{
        #[cfg(feature = "RV64")]
        if Xlen::xlen() >= 64 {
            Ok($e)
        } else {
            Err(())
        }

        #[cfg(not(feature = "RV64"))]
        Err(())
    }};
}

macro_rules! en_if_ext {
    ($e_id:ident, $e_str:literal, $self:ident, $e:expr) => {{
        #[cfg(feature = $e_str)]
        if $self.$e_id {
            Ok($e)
        } else {
            Err(())
        }
        #[cfg(not(feature = $e_str))]
        Err(())
    }};
}

macro_rules! en_if_ext_a {
    ($self:ident, $e:expr) => {
        en_if_ext!(A, "A", $self, $e)
    };
}

macro_rules! en_if_ext_c {
    ($self:ident, $e:expr) => {
        en_if_ext!(C, "C", $self, $e)
    };
}

macro_rules! en_if_ext_d {
    ($self:ident, $e:expr) => {
        en_if_ext!(D, "D", $self, $e)
    };
}

macro_rules! en_if_ext_f {
    ($self:ident, $e:expr) => {
        en_if_ext!(F, "F", $self, $e)
    };
}

macro_rules! en_if_ext_m {
    ($self:ident, $e:expr) => {
        en_if_ext!(M, "M", $self, $e)
    };
}

macro_rules! en_if_ext_zicsr {
    ($self:ident, $e:expr) => {
        en_if_ext!(Zicsr, "Zicsr", $self, $e)
    };
}

macro_rules! en_if_ext_zifencei {
    ($self:ident, $e:expr) => {
        en_if_ext!(Zifencei, "Zifencei", $self, $e)
    };
}
