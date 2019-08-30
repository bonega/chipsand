use crate::joypad::Joypad;
use crate::ppu::PPU;
use crate::serial::Serial;
use crate::timer::Timer;
use crate::{mbc, InputReceiver, ScreenSender};

const DMA_LENGTH: u16 = 160;

pub struct MMU {
    mbc: Box<dyn mbc::MBC>,
    wram: [u8; 8192],
    hram: [u8; 128],
    iram: [u8; 0x80],
    oam: [u8; 160],
    pub timer: Timer,
    pub interrupt_flags: u8,
    pub interrupt_enable: u8,
    dma_cycles_left: u16,
    dma_start_adr: u16,
    pub ppu: PPU,
    pub serial: Serial,
    pub joypad: Joypad,
}

impl MMU {
    pub fn new(rom: Vec<u8>, screen_sender: ScreenSender, input_receiver: InputReceiver) -> Self {
        MMU {
            mbc: mbc::load(rom),
            wram: [0; 8192],
            hram: [0; 128],
            iram: [0; 0x80],
            oam: [0; 160],
            timer: Timer::new(),
            interrupt_flags: 0,
            interrupt_enable: 0,
            dma_cycles_left: 0,
            dma_start_adr: 0,
            ppu: PPU::new(screen_sender),
            serial: Serial::new(),
            joypad: Joypad::new(input_receiver),
        }
    }

    pub fn get_interrupts(&self) -> u8 {
        self.interrupt_flags & self.interrupt_enable
    }

    fn start_dma(&mut self, high_adr: u8) {
        let adr = (high_adr as u16) << 8;
        //        println!("Dma request adr 0x{:X}", adr);
        self.dma_cycles_left = DMA_LENGTH * 4;
        self.dma_start_adr = adr;
    }

    pub fn read_word(&self, adr: u16) -> u8 {
        match adr {
            0x0000..=0x7FFF => self.mbc.read_word(adr),
            0xA000..=0xBFFF => self.mbc.read_word(adr),
            0xC000..=0xDFFF => self.wram[(adr - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(adr - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[adr as usize - 0xFE00],
            0xFEA0..=0xFEFF => 0x00, // Undocumented
            0xFF00 => self.joypad.read_word(),
            0xFF01..=0xFF02 => self.serial.read_word(adr),
            0xFF04..=0xFF07 => self.timer.read_word(adr),
            0xFF46 => return (self.dma_start_adr >> 8) as u8,
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4A => self.ppu.read_word(adr),
            0xFF01..=0xFF0E | 0xFF10..=0xFF7F => self.iram[(adr - 0xFF00) as usize],
            0xFF0F => self.interrupt_flags,
            0xFF80..=0xFFFE => self.hram[(adr - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
        }
    }

    pub fn read_dw(&self, adr: u16) -> u16 {
        let h = self.read_word(adr) as u16;
        let l = (self.read_word(adr.wrapping_add(1)) as u16) << 8;
        h + l
    }

    pub fn write_word(&mut self, adr: u16, val: u8) {
        match adr {
            0x0000..=0x7FFF => self.mbc.write_word(adr, val),
            0xA000..=0xBFFF => self.mbc.write_word(adr, val),
            0xC000..=0xDFFF => self.wram[(adr - 0xC000) as usize] = val,
            0xE000..=0xFDFF => self.wram[(adr - 0xE000) as usize] = val,
            0xFE00..=0xFE9F => self.oam[adr as usize - 0xFE00] = val,
            0xFEA0..=0xFEFF => {} //undocumented
            0xFF00 => self.joypad.write_word(val),
            0xFF01..=0xFF02 => self.serial.write_word(adr, val),
            0xFF04..=0xFF07 => self.timer.write_word(adr, val),
            0xFF46 => self.start_dma(val),
            0xFFFF => self.interrupt_enable = val,
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF45 | 0xFF47..=0xFF4A => {
                self.ppu.write_word(adr, val)
            }
            0xFF01..=0xFF0E | 0xFF10..=0xFF7F => self.iram[(adr - 0xFF00) as usize] = val,
            0xFF0F => self.interrupt_flags = val | 0b11100000,
            0xFF80..=0xFFFE => self.hram[(adr - 0xFF80) as usize] = val,
        }
    }

    pub fn write_dw(&mut self, adr: u16, val: u16) {
        self.write_word(adr, (val & 0xFF) as u8);
        self.write_word(adr.wrapping_add(1), (val >> 8) as u8);
    }

    pub fn tick(&mut self) {
        if self.dma_cycles_left > 0 {
            let offset = DMA_LENGTH - self.dma_cycles_left as u16 / 4;
            let source_adr = self.dma_start_adr + offset;
            let v = self.read_word(source_adr);
            self.write_word(0xFE00 + offset, v);
            self.dma_cycles_left -= 4;
        }
        let timer_interrupt = self.timer.tick();
        let ppu_ints = self.ppu.tick();
        let serial_ints = self.serial.tick();
        let joypad_ints = self.joypad.tick();
        self.joypad.process_inputs();
        self.interrupt_flags |= timer_interrupt as u8 | ppu_ints | serial_ints as u8 | joypad_ints;
    }
}
