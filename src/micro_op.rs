#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpCond {
    Eq,
    Ne,
    Lt,
    Ge,
    LtU,
    GeU,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemWidth {
    B,
    H,
    W,
    #[cfg(feature = "RV64")]
    D,
    BU,
    HU,
    #[cfg(feature = "RV64")]
    WU,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceMode {
    Normal,
    Tso,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscMemOp {
    /// pred & succ order
    Fence(u8, FenceMode),
    #[cfg(feature = "Zifencei")]
    FenceI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sll,
    Slt,
    SltU,
    Xor,
    Srl,
    Or,
    And,
    Sub,
    Sra,
    #[cfg(feature = "RV64")]
    AddW,
    #[cfg(feature = "RV64")]
    SllW,
    #[cfg(feature = "RV64")]
    SrlW,
    #[cfg(feature = "RV64")]
    SubW,
    #[cfg(feature = "RV64")]
    SraW,
    #[cfg(feature = "M")]
    Mul,
    #[cfg(feature = "M")]
    Mulh,
    #[cfg(feature = "M")]
    MulhU,
    #[cfg(feature = "M")]
    MulhSU,
    #[cfg(feature = "M")]
    Div,
    #[cfg(feature = "M")]
    DivU,
    #[cfg(feature = "M")]
    Rem,
    #[cfg(feature = "M")]
    RemU,
    #[cfg(all(feature = "M", feature = "RV64"))]
    MulW,
    #[cfg(all(feature = "M", feature = "RV64"))]
    DivW,
    #[cfg(all(feature = "M", feature = "RV64"))]
    DivUW,
    #[cfg(all(feature = "M", feature = "RV64"))]
    RemW,
    #[cfg(all(feature = "M", feature = "RV64"))]
    RemUW,
    #[cfg(feature = "A")]
    Second,
    #[cfg(feature = "A")]
    Max,
    #[cfg(feature = "A")]
    MaxU,
    #[cfg(feature = "A")]
    Min,
    #[cfg(feature = "A")]
    MinU,
    // #[cfg(all(feature = "A", feature = "RV64"))]
    // MaxW,
    // #[cfg(all(feature = "A", feature = "RV64"))]
    // MaxUW,
    // #[cfg(all(feature = "A", feature = "RV64"))]
    // MinW,
    // #[cfg(all(feature = "A", feature = "RV64"))]
    // MinUW,
}

#[cfg(feature = "A")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemOrder {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
}

#[cfg(feature = "Zicsr")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrOp {
    Rw,
    Rs,
    Rc,
    Rwi,
    Rsi,
    Rci,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    AddrMisalign(MemProtect),
    AccessFault(MemProtect),
    PageFault(MemProtect),
    IllegalInstr,
    Ecall,
    Ebreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemProtect {
    R,
    W,
    X,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    S,
    #[cfg(feature = "D")]
    D,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundMode {
    Rne,
    Rtz,
    Rdn,
    Rup,
    Rmm,
    Dyn,
    None,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpTenaryOp {
    MAdd,
    MSub,
    NMSub,
    NMAdd,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    SgnJ,
    SgnJN,
    SgnJX,
    Min,
    Max,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpUnaryOp {
    Sqrt,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpCmpCond {
    Eq,
    Lt,
    Le,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpGpOp {
    W,
    WU,
    #[cfg(feature = "RV64")]
    L,
    #[cfg(feature = "RV64")]
    LU,
    MV,
    Class,
}

#[cfg(feature = "F")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpFpOp {
    W,
    WU,
    #[cfg(feature = "RV64")]
    L,
    #[cfg(feature = "RV64")]
    LU,
    MV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    Undecoded,
    Nop,
    Trap(Exception),
    /// rd, rs1, imm
    OpImm(u8, u8, i32, BinaryOp),
    /// rd, rs1, rs2
    Op(u8, u8, u8, BinaryOp),
    /// rd, imm
    Auipc(u8, i32),
    /// rd, imm
    Lui(u8, i32),
    /// rd, rs1, offset
    Load(u8, u8, i32, MemWidth),
    /// rs1, rs2, offset
    Store(u8, u8, i32, MemWidth),
    MiscMem(MiscMemOp),
    /// rs1, rs2, offset
    Branch(u8, u8, i32, CmpCond),
    /// rd, offset
    Jal(u8, i32),
    /// rd, rs1, offset
    Jalr(u8, u8, i32),
    /// rd, rs1 / uimm, addr
    #[cfg(feature = "Zicsr")]
    Csr(u8, u8, u16, CsrOp),
    /// rd, rs1
    #[cfg(feature = "A")]
    LoadReserved(u8, u8, MemOrder, MemWidth),
    /// rd, rs1, rs2
    #[cfg(feature = "A")]
    StoreConditional(u8, u8, u8, MemOrder, MemWidth),
    /// rd, rs1, rs2
    #[cfg(feature = "A")]
    Amo(u8, u8, u8, MemOrder, MemWidth, BinaryOp),
    /// rd, rs1, imm
    #[cfg(feature = "F")]
    LoadFp(u8, u8, i32, Precision),
    /// rs1, rs2, imm
    #[cfg(feature = "F")]
    StoreFp(u8, u8, i32, Precision),
    /// rd, rs1, rs2, rs3
    #[cfg(feature = "F")]
    FpOp3(u8, u8, u8, u8, RoundMode, Precision, FpTenaryOp),
    /// rd, rs1, rs2
    #[cfg(feature = "F")]
    FpOp2(u8, u8, u8, RoundMode, Precision, FpBinaryOp),
    /// rd, s1
    #[cfg(feature = "F")]
    FpOp(u8, u8, RoundMode, Precision, FpUnaryOp),
    /// rd, rs1
    #[cfg(feature = "F")]
    FpCvtGp(u8, u8, RoundMode, Precision, FpGpOp),
    /// rd, rs1
    #[cfg(feature = "F")]
    GpCvtFp(u8, u8, RoundMode, Precision, GpFpOp),
    /// rd, rs1, rs2
    #[cfg(feature = "F")]
    FpCmp(u8, u8, u8, Precision, FpCmpCond),
    /// rd, rs1  from_precision, to_precision
    #[cfg(feature = "D")]
    FpCvtFp(u8, u8, RoundMode, Precision, Precision),
}
