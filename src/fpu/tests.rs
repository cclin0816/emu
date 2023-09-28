use super::*;

fn f32_op2_check(vals: &[u32], exp: &[(u32, u8)], op: FpBinaryOp) {
    let mut fpu = Fpu::default();
    let mut idx = 0usize;
    for &v1 in vals {
        for &v2 in vals {
            fpu.u32_mv_f32(0, v1);
            fpu.u32_mv_f32(1, v2);
            fpu.clr_all_fpe();
            fpu.binary_op(0, 0, 1, Precision::S, op);
            if exp[idx] != (fpu.f32_mv_u32(0), fpu.get_fpe().as_u8()) {
                println!(
                    "op: {:?}\nv1: {}\nv2: {}\nexp_val: {}\nexp_fpe: {}\nres_val: {}\nres_fpe: {}",
                    op,
                    v1,
                    v2,
                    exp[idx].0,
                    exp[idx].1,
                    fpu.f32_mv_u32(0),
                    fpu.get_fpe().as_u8()
                );
                panic!("f32_op2_check failed")
            }
            idx += 1;
        }
    }
}

#[test]
fn sanity() {
    let vals: [u32; 14] = [
        0xffc00000, 0xff800001, 0xff800000, 0xff7fffff, 0xbf800000, 0x80000001, 0x80000000,
        0x00000000, 0x00000001, 0x3f800000, 0x7f7fffff, 0x7f800000, 0x7f800001, 0x7fc00000,
    ];

    // add
    let exp: [(u32, u8); 196] = [
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x5),
        (0xff7fffff, 0x1),
        (0xff7fffff, 0x1),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x1),
        (0xff7fffff, 0x1),
        (0x0, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x1),
        (0xc0000000, 0x0),
        (0xbf800000, 0x1),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x1),
        (0x0, 0x0),
        (0x7f7fffff, 0x1),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x1),
        (0xbf800000, 0x1),
        (0x80000002, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x0, 0x0),
        (0x3f800000, 0x1),
        (0x7f7fffff, 0x1),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x0),
        (0xbf800000, 0x0),
        (0x80000001, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x1, 0x0),
        (0x3f800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x0),
        (0xbf800000, 0x0),
        (0x80000001, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x1, 0x0),
        (0x3f800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x1),
        (0xbf800000, 0x1),
        (0x0, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x2, 0x0),
        (0x3f800000, 0x1),
        (0x7f7fffff, 0x1),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x1),
        (0x0, 0x0),
        (0x3f800000, 0x1),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x1),
        (0x40000000, 0x0),
        (0x7f7fffff, 0x1),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0x0, 0x0),
        (0x7f7fffff, 0x1),
        (0x7f7fffff, 0x1),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x1),
        (0x7f7fffff, 0x1),
        (0x7f800000, 0x5),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::Add);
    // skip sub
    // mul
    let exp: [(u32, u8); 196] = [
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f800000, 0x5),
        (0x7f7fffff, 0x0),
        (0x34ffffff, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0xb4ffffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff800000, 0x5),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x3f800000, 0x0),
        (0x1, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000001, 0x0),
        (0xbf800000, 0x0),
        (0xff7fffff, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x34ffffff, 0x0),
        (0x1, 0x0),
        (0x0, 0x3),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x3),
        (0x80000001, 0x0),
        (0xb4ffffff, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xb4ffffff, 0x0),
        (0x80000001, 0x0),
        (0x80000000, 0x3),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x3),
        (0x1, 0x0),
        (0x34ffffff, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff7fffff, 0x0),
        (0xbf800000, 0x0),
        (0x80000001, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x1, 0x0),
        (0x3f800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x5),
        (0xff7fffff, 0x0),
        (0xb4ffffff, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x34ffffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f800000, 0x5),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::Mul);
    // div
    let exp: [(u32, u8); 196] = [
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x3f800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f800000, 0x5),
        (0x7f800000, 0x8),
        (0xff800000, 0x8),
        (0xff800000, 0x5),
        (0xff7fffff, 0x0),
        (0xbf800000, 0x0),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x200000, 0x3),
        (0x3f800000, 0x0),
        (0x7f800000, 0x5),
        (0x7f800000, 0x8),
        (0xff800000, 0x8),
        (0xff800000, 0x5),
        (0xbf800000, 0x0),
        (0x80200000, 0x3),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x0, 0x3),
        (0x1, 0x0),
        (0x3f800000, 0x0),
        (0x7f800000, 0x8),
        (0xff800000, 0x8),
        (0xbf800000, 0x0),
        (0x80000001, 0x0),
        (0x80000000, 0x3),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0x80000000, 0x3),
        (0x80000001, 0x0),
        (0xbf800000, 0x0),
        (0xff800000, 0x8),
        (0x7f800000, 0x8),
        (0x3f800000, 0x0),
        (0x1, 0x0),
        (0x0, 0x3),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0x80200000, 0x3),
        (0xbf800000, 0x0),
        (0xff800000, 0x5),
        (0xff800000, 0x8),
        (0x7f800000, 0x8),
        (0x7f800000, 0x5),
        (0x3f800000, 0x0),
        (0x200000, 0x3),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x80000000, 0x0),
        (0xbf800000, 0x0),
        (0xff7fffff, 0x0),
        (0xff800000, 0x5),
        (0xff800000, 0x8),
        (0x7f800000, 0x8),
        (0x7f800000, 0x5),
        (0x7f7fffff, 0x0),
        (0x3f800000, 0x0),
        (0x0, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x10),
        (0x7fc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::Div);
    // sgnj
    let exp: [(u32, u8); 196] = [
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::SgnJ);
    // sgnjn
    let exp: [(u32, u8); 196] = [
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::SgnJN);
    // sgnjx
    let exp: [(u32, u8); 196] = [
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x80000000, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x0, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x80000001, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0x1, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0xbf800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0x3f800000, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0xff7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0x7f7fffff, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0xff800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0x7f800000, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0xff800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0x7f800001, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0xffc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
        (0x7fc00000, 0x0),
    ];
    f32_op2_check(&vals, &exp, FpBinaryOp::SgnJX);
}
