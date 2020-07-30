use crate::Interrupt;

pub struct SC {
    sio_clk: bool,
    sio_en: bool,
}

impl SC {
    fn new() -> Self {
        SC {
            sio_clk: false,
            sio_en: false,
        }
    }

    pub fn read_word(&self) -> u8 {
        ((self.sio_en as u8) << 7) | 0b01111110 | self.sio_clk as u8
    }

    pub fn write_word(&mut self, v: u8) {
        self.sio_clk = (v & 1) != 0;
        self.sio_en = (v >> 7) != 0;
    }
}

pub struct Serial {
    pub sb: u8,
    sc: SC,
    counter: u16,
    sent: u8,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            sb: 0,
            sc: SC::new(),
            counter: 0,
            sent: 0,
        }
    }

    pub fn read_word(&self, adr: u16) -> u8 {
        match adr {
            0xFF01 => self.sb,
            0xFF02 => self.sc.read_word(),
            _ => panic!("Wrong read adr for Serial"),
        }
    }

    pub fn write_word(&mut self, adr: u16, v: u8) {
        match adr {
            0xFF01 => self.sb = v,
            0xFF02 => self.sc.write_word(v),
            _ => panic!("Wrong write adr for Serial"),
        }
    }

    pub fn tick(&mut self) -> Interrupt {
        if self.sc.sio_en {
            // TODO: implement support for serial
            // this is only provides a channel for writing
            if self.sc.sio_clk {
                self.counter += 1;
                if self.counter % 128 == 0 {
                    self.sent = (self.sent << 1) | (self.sb >> 7);
                    self.sb = (self.sb << 1) & 0xFF;
                    if self.counter == 8 * 128 {
                        self.counter = 0;
                        self.sent = 0;
                        self.sc.sio_en = false;
                        return Interrupt::SERIAL;
                    }
                }
            }
        }
        Interrupt::NoInterrupt
    }
}
