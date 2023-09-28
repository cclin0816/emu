use crate::{hart::Hart, uop::Exception, utils::Maybe, xlen::XlenT};

/// holds privlige state of hart  
/// eg. privilege level,
/// csr handler, ...
#[derive(Debug, Clone, Default)]
pub struct PrivCtrl {
    #[cfg(test)]
    pub hooked: bool,
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn raise(&mut self, _reason: Exception) -> Maybe<()> {
        #[cfg(test)]
        if self.priv_ctrl.hooked && _reason == Exception::Ebreak {
            self.stop_tok = true;
            return Err(());
        }
        todo!()
    }
    #[cfg(feature = "Zicsr")]
    pub fn csr_wr(&mut self, addr: u16, val: Xlen) -> Maybe<Xlen> {
        todo!()
    }
    #[cfg(feature = "Zicsr")]
    pub fn csr_set(&mut self, addr: u16, mask: Xlen) -> Maybe<Xlen> {
        todo!()
    }
    #[cfg(feature = "Zicsr")]
    pub fn csr_clr(&mut self, addr: u16, mask: Xlen) -> Maybe<Xlen> {
        todo!()
    }
}
