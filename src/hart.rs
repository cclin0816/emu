use crate::xlen::XlenT;

pub struct Hart<Xlen: XlenT> {
    pub rfs: RegFiles<Xlen>,
    pub pc: Xlen,
    pub isa: HartIsa,
}

#[repr(align(32))]
pub struct RegFiles<Xlen: XlenT> {
    gp: [Xlen; 32],
    #[cfg(feature = "F")]
    fp: [[u8; 8]; 32],
    #[cfg(feature = "V")]
    vec: [[u8; 32]; 32],
}

/// each flag corresponds to a feature in Cargo.toml
#[allow(non_snake_case)]
pub struct HartIsa {
    /// Atomic
    #[cfg(feature = "A")]
    pub A: bool,
    /// Compressed
    #[cfg(feature = "C")]
    pub C: bool,
    /// Double-precision floating-point
    #[cfg(feature = "D")]
    pub D: bool,
    /// RV32E
    #[cfg(feature = "E")]
    pub E: bool,
    /// Single-precision floating-point
    #[cfg(feature = "F")]
    pub F: bool,
    /// Integer Multiply/Divide
    #[cfg(feature = "M")]
    pub M: bool,
    /// CSR instructions
    #[cfg(feature = "Zicsr")]
    pub Zicsr: bool,
    /// Instruction-Fetch Fence
    #[cfg(feature = "Zifencei")]
    pub Zifencei: bool,
}

// macro_rules! impl_isa_check {
//     ($fname:ident, $flag:ident, $feature:literal) => {
//         pub fn $fname(&self) -> bool {
//             #[cfg(feature = $feature)]
//             {
//                 self.isa.$flag
//             }
//             #[cfg(not(feature = $feature))]
//             {
//                 false
//             }
//         }
//     };
// }

// // impl_isa_check!(A, C, D, E, F, M, Zicsr, Zifencei);
// #[allow(non_snake_case)]
// impl<Xlen: XlenT> Hart<Xlen> {
//     impl_isa_check!(has_A, A, "A");
//     impl_isa_check!(has_C, C, "C");
//     impl_isa_check!(has_D, D, "D");
//     impl_isa_check!(has_E, E, "E");
//     impl_isa_check!(has_F, F, "F");
//     impl_isa_check!(has_M, M, "M");
//     impl_isa_check!(has_Zicsr, Zicsr, "Zicsr");
//     impl_isa_check!(has_Zifencei, Zifencei, "Zifencei");
// }

// impl<Xlen: XlenT> Hart<Xlen> {
//     pub fn get_gp(&self, reg: u8) -> Xlen {
//         if reg == 0 {
//             Xlen::from(0)
//         } else {
//             self.gp_regs[reg as usize]
//         }
//     }
//     pub fn set_gp(&mut self, reg: u8, val: Xlen) {
//         self.gp_regs[reg as usize] = val;
//     }
//     pub fn add_pc<T>(&mut self, val: T)
//     where
//         Xlen: Cast<T>,
//     {
//         self.pc = self.pc.add(val);
//     }
//     pub fn exception(&mut self, reason: Exception) {
//         todo!()
//     }
//     pub fn interrupt(&mut self) {
//         todo!()
//     }
// }
