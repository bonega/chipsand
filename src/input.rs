use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::joypad;
use crate::joypad::{Joypad, Key};

#[derive(Debug, PartialEq)]
pub enum Control {
    KeyUp(joypad::Key),
    KeyDown(joypad::Key),
    Quit,
}

fn keycode_to_key(keycode: Keycode) -> Option<joypad::Key> {
    match keycode {
        Keycode::A => Some(Key::A),
        Keycode::B => Some(joypad::Key::B),
        Keycode::Return => Some(joypad::Key::Select),
        Keycode::Space => Some(joypad::Key::Start),
        Keycode::Up => Some(joypad::Key::Up),
        Keycode::Down => Some(joypad::Key::Down),
        Keycode::Left => Some(joypad::Key::Left),
        Keycode::Right => Some(joypad::Key::Right),
        _ => None,
    }
}

pub fn from_sdl2_event(e: Event) -> Option<Control> {
    match e {
        Event::KeyUp { keycode, .. } => match keycode {
            Some(Keycode::Escape) => Some(Control::Quit),
            Some(k) => keycode_to_key(k).map(|k| Control::KeyUp(k)),
            None => None,
        },
        Event::KeyDown { keycode, .. } => keycode_to_key(keycode?).map(|k| Control::KeyDown(k)),
        _ => None,
    }
}
