use super::Interrupt;
pub struct Timer {
    big_div: u16,
    delayed_edge: bool,
    tac: u8,
    tima_reload: bool,
    pub tima: u8,
    pub tma: u8,
    timer_enabled: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            big_div: 0xABCC,
            delayed_edge: false,
            tac: 0,
            tima_reload: false,
            tima: 0,
            tma: 0,
            timer_enabled: false,
        }
    }

    pub fn tick(&mut self) -> Interrupt {
        let mut interrupt = Interrupt::NoInterrupt;
        if self.tima_reload {
            self.tima = self.tma;
            self.tima_reload = false;
            interrupt = Interrupt::TIMER;
        }
        self.big_div = self.big_div.wrapping_add(4);
        let bit = match self.tac {
            0b00 => 9, // 1024
            0b11 => 7, // 256
            0b10 => 5, // 64
            0b01 => 3, // 16
            _ => panic!("selected impossible bit in timer counter"),
        };
        let edge = self.big_div & 1 << bit != 0 && self.timer_enabled;
        if !edge && self.delayed_edge {
            self.delayed_edge = edge;
            let (sum, carry) = self.tima.overflowing_add(1);
            if carry {
                self.tima_reload = true;
                self.tima = 0;
            } else {
                self.tima = sum;
            }
        } else {
            self.delayed_edge = edge;
        }
        return interrupt;
    }

    pub fn read_word(&self, adr: u16) -> u8 {
        match adr {
            0xFF04 => (self.big_div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | ((self.timer_enabled as u8) << 2) | 0b11111000,
            _ => panic!("Timer wrong read adr"),
        }
    }

    pub fn write_word(&mut self, adr: u16, v: u8) {
        match adr {
            0xFF04 => self.big_div = 0,
            0xFF05 => {
                self.tima_reload = false;
                self.tima = v
            }
            0xFF06 => self.tma = v,
            0xFF07 => {
                self.tac = v & 0b11;
                self.timer_enabled = (v & 0b100) != 0
            }
            _ => panic!("Timer wrong write adr"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = Timer::new();
        timer.write_word(0xFF07, 0b101);
        assert_eq!(timer.timer_enabled, true);
        assert_eq!(timer.read_word(0xFF07), 0b11111101);
        timer.write_word(0xFF07, 0b0);
        assert_eq!(timer.timer_enabled, false);
        timer.write_word(0xFF07, 0b111);
    }
}
