use crate::{hart::Hart, utils::Maybe, xlen::XlenT};

mod alu;
mod dispatch;

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn run(&mut self) {
        while self.stop_tok {
            let _ = self.exec_cycle();
        }
    }
    fn exec_cycle(&mut self) -> Maybe<()> {
        let ins = self.fetch_uop()?;
        ins.exec(self)
    }
}
