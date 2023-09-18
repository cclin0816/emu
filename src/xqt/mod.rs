mod alu;

// impl Instr {
//     pub fn exec<Xlen: XlenT>(&mut self, hart: &mut Hart<Xlen>) {
//         match *self {
//             Instr::RdRs1Rs2(rd, rs1, rs2, op) => {
//                 let lhs = hart.get_gp(rs1);
//                 let rhs = hart.get_gp(rs2);
//                 let res = op.exec(lhs, rhs);
//                 hart.set_gp(rd, res);
//                 hart.add_pc(4);
//             }
//             Instr::RdRs1Imm(rd, rs1, imm, op) => {
//                 let lhs = hart.get_gp(rs1);
//                 let rhs = Xlen::from(imm);
//                 let res = op.exec(lhs, rhs);
//                 hart.set_gp(rd, res);
//                 hart.add_pc(4);
//             }
//             Instr::BranchCond(rs1, rs2, offset, cond) => {
//                 let lhs = hart.get_gp(rs1);
//                 let rhs = hart.get_gp(rs2);
//                 if cond.test(lhs, rhs) {
//                     hart.add_pc(offset);
//                 } else {
//                     hart.add_pc(4);
//                 }
//             }
//             Instr::Jal(rd, offset) => {
//                 hart.set_gp(rd, hart.pc.add(4));
//                 hart.add_pc(offset);
//             }
//             Instr::Jalr(rd, rs1, offset) => {
//                 let addr = hart.get_gp(rs1).add(offset);
//                 hart.set_gp(rd, hart.pc.add(4));
//                 hart.pc = addr;
//             }
//             Instr::Lui(rd, imm) => {
//                 hart.set_gp(rd, Xlen::from(imm));
//                 hart.add_pc(4);
//             }
//             Instr::Auipc(rd, imm) => {
//                 let addr = hart.pc.add(imm);
//                 hart.set_gp(rd, addr);
//                 hart.add_pc(4);
//             }
//             _ => todo!(),
//         }
//     }
// }
