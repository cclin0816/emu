use crate::{fpu::Fpu, xlen::XlenT};

#[repr(align(32))]
#[derive(Debug, Clone)]
pub struct Hart<Xlen: XlenT, const EMB: bool> {
    gp: [Xlen; 32],
    #[cfg(feature = "F")]
    pub fpu: Fpu<Xlen>,
    pub pc: Xlen,
    pub isa: HartIsa<Xlen>,
}

/// each flag corresponds to a feature in Cargo.toml\
/// act as a fast query misa register
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default)]
pub struct HartIsa<Xlen: XlenT> {
    /// Atomic
    #[cfg(feature = "A")]
    pub A: bool,
    /// Compressed
    #[cfg(feature = "C")]
    pub C: bool,
    /// Double-precision floating-point
    #[cfg(feature = "D")]
    pub D: bool,
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

    xlen: std::marker::PhantomData<Xlen>,
}

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
