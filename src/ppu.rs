use crate::{Interrupt, Pixel, ScreenSender};
use std::collections::VecDeque;

pub struct Control {
    lcd_en: bool,
    win_map: bool,
    win_en: bool,
    tile_sel: bool,
    bg_map: bool,
    obj_size: bool,
    obj_en: bool,
    bg_en: bool,
}

impl Control {
    fn new() -> Self {
        Control {
            lcd_en: false,
            win_map: false,
            win_en: false,
            tile_sel: false,
            bg_map: false,
            obj_size: false,
            obj_en: false,
            bg_en: false,
        }
    }

    fn read_word(&self) -> u8 {
        let mut flags = 0;
        if self.lcd_en {
            flags |= 0b10000000;
        }
        if self.win_map {
            flags |= 0b01000000;
        }
        if self.win_en {
            flags |= 0b00100000;
        }
        if self.tile_sel {
            flags |= 0b00010000;
        }
        if self.bg_map {
            flags |= 0b00001000;
        }
        if self.obj_size {
            flags |= 0b00000100;
        }
        if self.obj_en {
            flags |= 0b00000010;
        }
        if self.bg_en {
            flags |= 0b00000001;
        }
        flags
    }

    fn write_word(&mut self, v: u8) {
        if v & 0b10000000 != 0 && self.lcd_en == false {
            println!("lcd enable");
        }
        if v & 0b10000000 == 0 && self.lcd_en == true {
            println!("lcd disable");
        }
        self.lcd_en = (v & 0b10000000) != 0;
        self.win_map = (v & 0b01000000) != 0;
        self.win_en = (v & 0b00100000) != 0;
        self.tile_sel = (v & 0b00010000) != 0;
        if v & 0b00010000 == 0 {
            println!("disabling tilesel 0x{:X}", v);
        }
        self.bg_map = (v & 0b00001000) != 0;
        self.obj_size = (v & 0b00000100) != 0;
        self.obj_en = (v & 0b00000010) != 0;
        self.bg_en = (v & 0b00000001) != 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    TRANSFER = 3,
}

pub struct Stat {
    int_lyc: bool,
    int_oam: bool,
    int_vblank: bool,
    int_hblank: bool,
    coincidence_flag: bool,
    pub mode: Mode,
}

impl Stat {
    fn new() -> Self {
        Stat {
            int_lyc: false,
            int_oam: false,
            int_vblank: false,
            int_hblank: false,
            coincidence_flag: false,
            mode: Mode::OAM,
        }
    }

    pub fn write_word(&mut self, v: u8) {
        self.int_lyc = v & 0b01000000 != 0;
        self.int_oam = v & 0b00100000 != 0;
        self.int_vblank = v & 0b00010000 != 0;
        self.int_hblank = v & 0b00001000 != 0;
    }

    pub fn read_word(&self) -> u8 {
        let mut flags = 0b10000000;
        if self.int_lyc {
            flags |= 0b01000000;
        }
        if self.int_oam {
            flags |= 0b00100000;
        }
        if self.int_vblank {
            flags |= 0b00010000;
        }
        if self.int_hblank {
            flags |= 0b00001000;
        }
        if self.coincidence_flag {
            flags |= 0b00000100;
        }
        println!("flags 0x{:X}", flags);
        flags | self.mode as u8
    }
}

const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const OAM_SIZE: usize = 0xFE9F - 0xFE00 + 1;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Fetcher {
    tile_index: u8,
    tile_map_adr: usize,
    high_byte: u8,
    low_byte: u8,
    pixel_row: [u8; 8],
    state: FetcherStates,
    y_offset: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FetcherStates {
    TileIndex,
    HighByte,
    LowByte,
    Idle,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            tile_index: 0,
            tile_map_adr: 0,
            high_byte: 0,
            low_byte: 0,
            pixel_row: [0; 8],
            state: FetcherStates::TileIndex,
            y_offset: 0,
        }
    }

    fn merge_bytes(&mut self, bgp: u8) {
        let h = self.high_byte;
        let l = self.low_byte;
        for (x, elem) in self.pixel_row.iter_mut().enumerate() {
            let v = (((l >> (7 - x as u8)) & 1) << 1) + ((h >> (7 - x as u8)) & 1);
            let shade = (bgp >> (v * 2)) & 0b11;
            *elem = shade;
        }
    }

    fn flush_row(&mut self, fifo: &mut PixelFifo) {
        for i in self.pixel_row.iter() {
            fifo.push(*i);
        }
        self.tile_map_adr += 1;
    }

    pub fn reset(&mut self, bg_map: bool, x: u8, y: u8) {
        let base_adr: usize = if bg_map { 0x1C00 } else { 0x1800 };
        self.tile_map_adr = base_adr + (y as usize / 8 * 32) + x as usize / 8;
        self.state = FetcherStates::TileIndex;
        self.y_offset = y % 8 * 2;
    }

    fn get_tile_adr(&self, tile_sel: bool) -> usize {
        let tile_index = self.tile_index as usize;
        let adr = if tile_sel {
            if tile_index < 0x80 {
                0x000 + tile_index * 16
            } else {
                0x800 + (tile_index - 0x80) * 16
            }
        } else {
            if tile_index < 0x80 {
                0x1000 + tile_index * 16
            } else {
                0x800 + (tile_index - 0x80) * 16
            }
        };
        adr as usize + self.y_offset as usize
    }

    pub fn tick(&mut self, vram: &[u8; 8192], tile_sel: bool, bgp: u8, pixel_fifo: &mut PixelFifo) {
        self.state = match self.state {
            FetcherStates::TileIndex => {
                self.tile_index = vram[self.tile_map_adr];
                FetcherStates::HighByte
            }
            FetcherStates::HighByte => {
                let adr = self.get_tile_adr(tile_sel);
                self.high_byte = vram[adr];
                FetcherStates::LowByte
            }
            FetcherStates::LowByte => {
                let adr = self.get_tile_adr(tile_sel);
                self.low_byte = vram[adr + 1];
                self.merge_bytes(bgp);
                if !pixel_fifo.is_ready() {
                    self.flush_row(pixel_fifo);
                    FetcherStates::TileIndex
                } else {
                    FetcherStates::Idle
                }
            }
            FetcherStates::Idle => {
                self.flush_row(pixel_fifo);
                FetcherStates::TileIndex
            }
        }
    }
}

pub struct PPU {
    control: Control,
    pub lcd_stat: Stat,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub wy: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub vram: [u8; VRAM_SIZE],
    pub oam: [u8; OAM_SIZE],
    cycles_elapsed: u16,
    screen_sender: ScreenSender,
    pixel_fifo: PixelFifo,
    fetcher: Fetcher,
    is_state_enter: bool,
}

impl PPU {
    pub fn new(screen_sender: ScreenSender) -> Self {
        let ppu = PPU {
            control: Control::new(),
            lcd_stat: Stat::new(),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            cycles_elapsed: 0,
            screen_sender,
            pixel_fifo: PixelFifo::new(),
            fetcher: Fetcher::new(),
            is_state_enter: false,
        };
        ppu
    }

    fn set_next_state(&mut self, state: Mode) {
        self.is_state_enter = true;
        self.lcd_stat.mode = state;
    }

    fn is_state_enter(&mut self) -> bool {
        let res = self.is_state_enter;
        self.is_state_enter = false;
        res
    }

    pub fn tick(&mut self) -> u8 {
        if !self.control.lcd_en {
            return Interrupt::NoInterrupt as u8;
        }
        let mut interrupt = 0;
        self.cycles_elapsed += 4;
        match self.lcd_stat.mode {
            Mode::OAM => {
                if self.is_state_enter() && self.lcd_stat.int_oam {
                    interrupt = Interrupt::LCDStat as u8;
                } else if self.cycles_elapsed == 80 {
                    self.set_next_state(Mode::TRANSFER);
                }
            }
            Mode::TRANSFER => {
                if self.is_state_enter() {
                    self.fetcher.reset(self.control.bg_map, self.scx, self.ly);
                    self.pixel_fifo.reset(self.scx);
                }
                self.fetcher.tick(
                    &self.vram,
                    self.control.tile_sel,
                    self.bgp,
                    &mut self.pixel_fifo,
                );
                self.fetcher.tick(
                    &self.vram,
                    self.control.tile_sel,
                    self.bgp,
                    &mut self.pixel_fifo,
                );
                self.pixel_fifo.tick(self.ly);
                self.pixel_fifo.tick(self.ly);
                self.pixel_fifo.tick(self.ly);
                self.pixel_fifo.tick(self.ly);

                if self.pixel_fifo.row_is_done() {
                    self.pixel_fifo.empty_queue();
                    self.ly += 1;

                    self.set_next_state(Mode::HBlank);
                }
            }
            Mode::HBlank => {
                if self.is_state_enter() && self.lcd_stat.int_hblank {
                    interrupt = Interrupt::LCDStat as u8;
                } else if self.cycles_elapsed == 456 {
                    self.cycles_elapsed = 0;
                    self.set_next_state(match self.ly {
                        144 => Mode::VBlank,
                        _ => Mode::OAM,
                    });
                }
            }
            Mode::VBlank => {
                if self.is_state_enter() {
                    self.screen_sender.send(self.pixel_fifo.screen);
                    interrupt |= Interrupt::VBLANK as u8;
                    if self.lcd_stat.int_vblank || self.lcd_stat.int_oam {
                        interrupt |= Interrupt::LCDStat as u8;
                    }
                } else if self.cycles_elapsed % 456 == 0 {
                    self.ly += 1;
                    if self.ly == 154 {
                        self.ly = 0;
                        self.cycles_elapsed = 0;
                        self.set_next_state(Mode::OAM);
                    }
                }
            }
        }

        self.lcd_stat.coincidence_flag = self.ly == self.lyc;
        if self.lcd_stat.int_lyc && self.lcd_stat.coincidence_flag {
            interrupt |= Interrupt::LCDStat as u8;
        }
        interrupt
    }

    pub fn read_word(&self, adr: u16) -> u8 {
        match adr {
            0x8000..=0x9FFF => {
                if self.lcd_stat.mode != Mode::TRANSFER || !self.control.lcd_en {
                    self.vram[adr as usize - 0x8000]
                } else {
                    0xFF
                }
            }
            0xFE00..=0xFE9F => {
                if self.lcd_stat.mode != Mode::TRANSFER && self.lcd_stat.mode != Mode::OAM
                    || !self.control.lcd_en
                {
                    self.oam[adr as usize - 0xFE00]
                } else {
                    0xFF
                }
            }
            0xFF40 => self.control.read_word(),
            0xFF41 => dbg!(self.lcd_stat.read_word()),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            _ => panic!("No such ppu adr 0x{:X}", adr),
        }
    }

    pub fn write_word(&mut self, adr: u16, v: u8) {
        match adr {
            0x8000..=0x9FFF => {
                if self.lcd_stat.mode != Mode::TRANSFER || !self.control.lcd_en {
                    self.vram[adr as usize - 0x8000] = v;
                    if adr > 0x9800 && adr < 0x9A00 {
                        //                    println!("write tile {} to mem 0x{:X}", v, adr);
                    }
                }
            }
            0xFE00..=0xFE9F => {
                if self.lcd_stat.mode != Mode::TRANSFER && self.lcd_stat.mode != Mode::OAM
                    || !self.control.lcd_en
                {
                    self.oam[adr as usize - 0xFE00] = v;
                }
            }
            0xFF40 => self.control.write_word(v),
            0xFF41 => self.lcd_stat.write_word(v),
            0xFF42 => self.scy = v,
            0xFF43 => self.scx = v,
            0xFF44 => {} // read only
            0xFF45 => self.lyc = v,
            0xFF47 => self.bgp = v,
            0xFF48 => self.obp0 = v,
            0xFF49 => self.obp1 = v,
            0xFF4A => self.wy = v,
            _ => {}
        }
    }
}

pub struct PixelFifo {
    queue: VecDeque<u8>,
    pub screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    x: usize,
    scx: u8,
}

impl PixelFifo {
    fn new() -> Self {
        PixelFifo {
            queue: VecDeque::with_capacity(16),
            screen: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            x: 0,
            scx: 0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.queue.len() > 8
    }
    pub fn push(&mut self, v: u8) {
        self.queue.push_back(v);
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.queue.pop_front()
    }

    fn reset(&mut self, scx: u8) {
        self.queue.clear();
        self.scx = scx % 8;
        self.x = 0;
    }

    fn row_is_done(&self) -> bool {
        self.x == 160
    }

    fn empty_queue(&mut self) {
        self.queue.clear();
    }

    pub fn tick(&mut self, y: u8) {
        if self.is_ready() {
            let v = self.pop().unwrap();
            if self.scx > 0 {
                self.scx -= 1;
            } else {
                self.screen[y as usize][self.x] = v;
                self.x += 1;
            }
        }
    }
}
