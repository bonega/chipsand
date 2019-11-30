use std::fs;
use std::sync::mpsc::{Receiver, SyncSender};

use crate::input::Control;

pub mod cpu;
pub mod display;
pub mod input;
pub mod joypad;
pub mod mbc;
pub mod mmu;
pub mod ppu;
pub mod registers;
pub mod serial;
pub mod timer;

pub type Pixel = u8;
pub type ScreenBuffer = [[u8; 160]; 144];
pub const BLANK_SCREEN: ScreenBuffer = [[0; 160]; 144];
pub type ScreenSender = SyncSender<[[Pixel; 160]; 144]>;
pub type InputReceiver = Receiver<Control>;

pub const CPU_CLOCK: u32 = 4194304;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interrupt {
    VBLANK = 0b00001,
    LCDStat = 0b00010,
    TIMER = 0b00100,
    SERIAL = 0b01000,
    JOYPAD = 0b10000,
    NoInterrupt = 0b00000,
}

pub fn screen_buffer_to_vec(pixels: &ScreenBuffer) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|array| array.iter())
        .cloned()
        .collect()
}

pub fn save_screen_buffer(pixels: &ScreenBuffer, path: String) {
    let pixs = screen_buffer_to_vec(pixels);
    let mut buffer = fs::File::create(path);
    serde_json::to_writer(buffer.unwrap(), &pixs);
}
