use std::default;

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
    /// (pred, succ, ...)
    Fence(u8, u8, FenceMode),
    #[cfg(feature = "Zifencei")]
    FenceI,
}

impl MiscMemOp {
    pub const FENCE_W: u8 = 1 << 0;
    pub const FENCE_R: u8 = 1 << 1;
    pub const FENCE_I: u8 = 1 << 2;
    pub const FENCE_O: u8 = 1 << 3;
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
    /// used for amo swap
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
pub enum FpTernaryOp {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Instr {
    #[default]
    Undecoded,

    Nop,
    Trap(Exception),
    /// (gp-rd, gp-rs1, imm, ...)
    OpImm(u8, u8, i32, BinaryOp),
    /// (gp-rd, gp-rs1, gp-rs2, ...)
    Op(u8, u8, u8, BinaryOp),
    /// (gp-rd, imm)
    Auipc(u8, i32),
    /// (gp-rd, gp-rs1, offset, ...)
    Load(u8, u8, i32, MemWidth),
    /// (gp-rs1, gp-rs2, offset, ...)
    Store(u8, u8, i32, MemWidth),
    MiscMem(MiscMemOp),
    /// (gp-rs1, gp-rs2, offset, ...)
    Branch(u8, u8, i32, CmpCond),
    /// (gp-rd, offset, ...)
    Jal(u8, i32),
    /// (gp-rd, gp-rs1, offset, ...)
    Jalr(u8, u8, i32),
    /// (gp-rd, gp-rs1 / uimm, csr_addr, ...)
    #[cfg(feature = "Zicsr")]
    Csr(u8, u8, u16, CsrOp),

    /// (gp-rd, gp-rs1, ...)
    #[cfg(feature = "A")]
    LoadReserved(u8, u8, MemOrder, MemWidth),
    /// (gp-rd, gp-rs1, gp-rs2, ...)
    #[cfg(feature = "A")]
    StoreConditional(u8, u8, u8, MemOrder, MemWidth),
    /// (gp-rd, gp-rs1, gp-rs2, ...)
    #[cfg(feature = "A")]
    Amo(u8, u8, u8, MemOrder, MemWidth, BinaryOp),

    /// (fp-rd, gp-rs1, offset, ...)
    #[cfg(feature = "F")]
    LoadFp(u8, u8, i32, Precision),
    /// (gp-rs1, fp-rs2, offset, ...)
    #[cfg(feature = "F")]
    StoreFp(u8, u8, i32, Precision),
    /// (fp-rd, fp-rs1, fp-rs2, fp-rs3, ...)
    #[cfg(feature = "F")]
    FpOp3(u8, u8, u8, u8, RoundMode, Precision, FpTernaryOp),
    /// (fp-rd, fp-rs1, fp-rs2, ...)
    #[cfg(feature = "F")]
    FpOp2(u8, u8, u8, RoundMode, Precision, FpBinaryOp),
    /// (fp-rd, fp-rs1, ...)
    #[cfg(feature = "F")]
    FpOp(u8, u8, RoundMode, Precision, FpUnaryOp),
    /// (gp-rd, fp-rs1, ...)
    #[cfg(feature = "F")]
    FpCvtGp(u8, u8, RoundMode, Precision, FpGpOp),
    /// (fp-rd, gp-rs1, ...)
    #[cfg(feature = "F")]
    GpCvtFp(u8, u8, RoundMode, Precision, GpFpOp),
    /// (gp-rd, fp-rs1, fp-rs2, ...)
    #[cfg(feature = "F")]
    FpCmp(u8, u8, u8, Precision, FpCmpCond),
    /// (fp-rd, fp-rs1, round_mode, from_precision, to_precision)
    #[cfg(feature = "D")]
    FpCvtFp(u8, u8, RoundMode, Precision, Precision),

    #[cfg(feature = "C")]
    CTrap(Exception),
    /// (gp-rd, gp-rs1, imm, ...)
    #[cfg(feature = "C")]
    COpImm(u8, u8, i32, BinaryOp),
    /// (gp-rd & gp-rs1, gp-rs2, ...)
    #[cfg(feature = "C")]
    COp(u8, u8, BinaryOp),
    /// (gp-rd, gp-rs1, offset, ...)
    #[cfg(feature = "C")]
    CLoad(u8, u8, i32, MemWidth),
    /// (gp-rs1, gp-rs2, offset, ...)
    #[cfg(feature = "C")]
    CStore(u8, u8, i32, MemWidth),
    /// (gp-rs1, offset, ...)
    #[cfg(feature = "C")]
    CBranch(u8, i32, CmpCond),
    /// (gp-rd, offset)
    #[cfg(feature = "C")]
    CJal(u8, i32),
    /// (gp-rd, gp-rs1)
    #[cfg(feature = "C")]
    CJalr(u8, u8),
    /// (gp-rd, fp-rs1, offset, ...)
    #[cfg(all(feature = "C", feature = "F"))]
    CLoadFp(u8, u8, i32, Precision),
    /// (gp-rs1, fp-rs2, offset, ...)
    #[cfg(all(feature = "C", feature = "F"))]
    CStoreFp(u8, u8, i32, Precision),
}
