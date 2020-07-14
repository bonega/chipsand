use crate::input::Control;
use crate::{InputReceiver, Interrupt};
use std::sync::mpsc::TryRecvError;

#[derive(Debug, PartialEq)]
pub enum Key {
    Down,
    Up,
    Left,
    Right,
    Start,
    Select,
    A,
    B,
}

pub struct Joypad {
    select_map: u8, // bitmap 0b10 = directions, 0b01 = buttons
    directions: u8, // bitmap show not pressed
    buttons: u8,
    input_receiver: InputReceiver,
    prev_state: u8,
}

const UP_MASK: u8 = 0b1000;
const DOWN_MASK: u8 = 0b0100;
const LEFT_MASK: u8 = 0b0010;
const RIGHT_MASK: u8 = 0b0001;
const START_MASK: u8 = 0b1000;
const SELECT_MASK: u8 = 0b0100;
const B_MASK: u8 = 0b0010;
const A_MASK: u8 = 0b0001;

impl Joypad {
    pub fn new(input_receiver: InputReceiver) -> Self {
        Joypad {
            select_map: 0x00,
            directions: 0b1111,
            buttons: 0b1111,
            input_receiver,
            prev_state: 0,
        }
    }

    pub fn read_word(&self) -> u8 {
        let mut res = 0b11000000;
        res |= self.select_map << 4;
        if self.select_map & 0b10 == 0 {
            res |= self.buttons;
        }
        if self.select_map & 0b01 == 0 {
            res |= self.directions;
        }
        res
    }

    pub fn write_word(&mut self, v: u8) {
        self.select_map = (v & 0b00110000) >> 4;
    }

    pub fn key_up(&mut self, key: Key) {
        use Key::*;
        match key {
            Up => self.directions |= UP_MASK,
            Down => self.directions |= DOWN_MASK,
            Left => self.directions |= LEFT_MASK,
            Right => self.directions |= RIGHT_MASK,
            Start => self.buttons |= START_MASK,
            Select => self.buttons |= SELECT_MASK,
            B => self.buttons |= B_MASK,
            A => self.buttons |= A_MASK,
        }
    }

    pub fn key_down(&mut self, key: Key) {
        use Key::*;
        match key {
            Up => self.directions &= !UP_MASK,
            Down => self.directions &= !DOWN_MASK,
            Left => self.directions &= !LEFT_MASK,
            Right => self.directions &= !RIGHT_MASK,
            Start => self.buttons &= !START_MASK,
            Select => self.buttons &= !SELECT_MASK,
            B => self.buttons &= !B_MASK,
            A => self.buttons &= !A_MASK,
        }
    }

    pub fn process_inputs(&mut self) {
        let res = self.input_receiver.try_recv();
        match res {
            Ok(control) => match control {
                Control::KeyUp(k) => self.key_up(k),
                Control::KeyDown(k) => self.key_down(k),
                _ => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Disconnected channel"),
        }
    }

    pub fn tick(&mut self) -> u8 {
        let pre = self.prev_state & 0b1111;
        self.prev_state = self.read_word();
        let new = self.prev_state & 0b1111;
        if pre & !new != 0 {
            Interrupt::JOYPAD as u8
        } else {
            Interrupt::NoInterrupt as u8
        }
    }
}
