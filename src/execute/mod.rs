use crate::{hart::Hart, utils::Maybe, xlen::XlenT};

mod alu;
mod dispatch;

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn run(&mut self) {
        while !self.stop_tok {
            let _ = self.exec_cycle();
        }
    }
    fn exec_cycle(&mut self) -> Maybe<()> {
        let ins = self.fetch_uop()?;
        ins.exec(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity() {
        let mut hart = Hart::<u32>::default();
        hart.priv_ctrl.hooked = true;
        // fib(20)
        let prog: [u8; 104] = [
            19, 5, 64, 1, 239, 0, 128, 0, 115, 0, 16, 0, 19, 1, 1, 255, 35, 38, 17, 0, 35, 36,
            129, 0, 35, 34, 145, 0, 35, 32, 33, 1, 19, 4, 5, 0, 147, 5, 32, 0, 19, 5, 16, 0, 99,
            98, 180, 2, 147, 4, 0, 0, 19, 9, 16, 0, 19, 5, 244, 255, 239, 240, 31, 253, 19, 4, 228,
            255, 179, 4, 149, 0, 227, 104, 137, 254, 19, 133, 20, 0, 131, 32, 193, 0, 3, 36, 129,
            0, 131, 36, 65, 0, 3, 41, 1, 0, 19, 1, 1, 1, 103, 128, 0, 0,
        ];
        let mut mem_region = Box::new([0u8; 16384]);
        mem_region[..104].copy_from_slice(&prog);
        hart.mem.hook_mem = Some(mem_region);
        hart.gprs[2] = 16384;
        hart.pc = 0;
        hart.run();
        assert_eq!(hart.gprs[10], 10946);
    }
}
