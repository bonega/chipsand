pub mod mbc0;
pub mod mbc1;

pub trait MBC {
    fn read_word(&self, adr: u16) -> u8;
    fn write_word(&mut self, adr: u16, val: u8);
}

pub fn load(rom: Vec<u8>) -> Box<dyn MBC> {
    Box::new(mbc0::MBC0::new(rom))
}
