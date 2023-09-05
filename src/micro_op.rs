#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CmpCond {
    Eq = 0,
    Ne = 1,
    Lt = 4,
    Ge = 5,
    LtU = 6,
    GeU = 7,
}

#[derive(Debug, Clone, Copy)]
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
    #[cfg(feature = "RV128")]
    DU,
    #[cfg(feature = "RV128")]
    Q,
}

#[derive(Debug, Clone, Copy)]
pub enum MiscMemOps {
    /// fence mode, pred & succ
    Fence(u8, u8),
    #[cfg(feature = "Zifencei")]
    FenceI,
}

#[derive(Debug, Clone, Copy)]
pub enum Ops {
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
    // future 128
    // bit 25 as +32
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
    #[cfg(feature = "A")]
    MaxW,
    #[cfg(feature = "A")]
    MaxUW,
    #[cfg(feature = "A")]
    MinW,
    #[cfg(feature = "A")]
    MinUW,
}

#[cfg(feature = "A")]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MemOrder {
    Relaxed = 0,
    Release = 1,
    Acquire = 2,
    AcqRel = 3,
}

#[cfg(feature = "Zicsr")]
#[derive(Debug, Clone, Copy)]
pub enum CsrOp {
    Rw,
    Rs,
    Rc,
    Rwi,
    Rws,
    Rwc,
}

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    Undecoded,
    Invalid,
    Reserved,
    Hint,
    Nop,
    Ecall,
    Ebreak,
    /// rd, rs1, imm
    OpImm(u8, u8, i32, Ops),
    /// rd, rs1, rs2
    Op(u8, u8, u8, Ops),
    /// rd, imm
    Auipc(u8, i32),
    /// rd, imm
    Lui(u8, i32),
    /// rd, rs1, offset
    Load(u8, u8, i32, MemWidth),
    /// rs1, rs2, offset
    Store(u8, u8, i32, MemWidth),
    MiscMem(MiscMemOps),
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
    Amo(u8, u8, u8, MemOrder, MemWidth, Ops),
}