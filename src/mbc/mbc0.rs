use crate::mbc::MBC;

pub struct MBC0 {
    rom: Vec<u8>,
    ext_ram: [u8; 8192],
}

impl MBC0 {
    pub fn new(rom: Vec<u8>) -> Self {
        MBC0 {
            rom,
            ext_ram: [0; 8192],
        }
    }
}

impl MBC for MBC0 {
    fn read_word(&self, adr: u16) -> u8 {
        match adr {
            0x0000..=0x7FFF => self.rom[adr as usize],
            0xA000..=0xBFFF => self.ext_ram[adr as usize - 0xA000],
            _ => panic!("No such adr 0x{:X} in mbc", adr),
        }
    }
    fn write_word(&mut self, adr: u16, val: u8) {
        match adr {
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF => self.ext_ram[adr as usize - 0xA000] = val,
            _ => panic!("No such adr 0x{:X} in mbc", adr),
        }
    }
}
