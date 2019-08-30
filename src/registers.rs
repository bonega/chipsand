#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    h: u8,
    l: u8,
    pub z_flag: bool,
    pub n_flag: bool,
    pub h_flag: bool,
    pub c_flag: bool,
    pub sp: u16,
    pub pc: u16,
}

pub trait RegIO<T, V> {
    fn write(&mut self, reg: T, v: V);
    fn read(&self, reg: T) -> V;
}

impl RegIO<R8, u8> for Registers {
    fn write(&mut self, reg: R8, v: u8) {
        match reg {
            R8::A => self.a = v,
            R8::F => unimplemented!(),
            R8::B => self.b = v,
            R8::C => self.c = v,
            R8::D => self.d = v,
            R8::E => self.e = v,
            R8::H => self.h = v,
            R8::L => self.l = v,
        }
    }

    fn read(&self, reg: R8) -> u8 {
        match reg {
            R8::A => self.a,
            R8::F => self.get_reg_f(),
            R8::B => self.b,
            R8::C => self.c,
            R8::D => self.d,
            R8::E => self.e,
            R8::H => self.h,
            R8::L => self.l,
        }
    }
}

impl RegIO<R16, u16> for Registers {
    fn write(&mut self, reg: R16, v: u16) {
        match reg {
            R16::AF => self.set_reg_af(v),
            R16::BC => self.set_reg_bc(v),
            R16::DE => self.set_reg_de(v),
            R16::HL => self.set_reg_hl(v),
            R16::SP => self.sp = v,
        }
    }

    fn read(&self, reg: R16) -> u16 {
        match reg {
            R16::AF => self.get_reg_af(),
            R16::BC => self.get_reg_bc(),
            R16::DE => self.get_reg_de(),
            R16::HL => self.get_reg_hl(),
            R16::SP => self.sp,
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            z_flag: false,
            n_flag: false,
            h_flag: false,
            c_flag: false,
            sp: 0,
            pc: 0x100,
        }
    }

    pub fn get_reg_f(&self) -> u8 {
        (((self.z_flag as u8) << 3)
            + ((self.n_flag as u8) << 2)
            + ((self.h_flag as u8) << 1)
            + (self.c_flag as u8))
            << 4
    }

    pub fn get_reg_af(&self) -> u16 {
        to_u16(self.a, self.get_reg_f())
    }

    pub fn set_reg_af(&mut self, v: u16) {
        self.a = high_bits(v);
        let lower = low_bits(v);
        self.z_flag = (lower >> 7) & 1 != 0;
        self.n_flag = (lower >> 6) & 1 != 0;
        self.h_flag = (lower >> 5) & 1 != 0;
        self.c_flag = (lower >> 4) & 1 != 0;
    }

    pub fn get_reg_bc(&self) -> u16 {
        to_u16(self.b, self.c)
    }

    pub fn set_reg_bc(&mut self, v: u16) {
        self.b = high_bits(v);
        self.c = low_bits(v);
    }

    pub fn get_reg_de(&self) -> u16 {
        to_u16(self.d, self.e)
    }

    pub fn set_reg_de(&mut self, v: u16) {
        self.d = high_bits(v);
        self.e = low_bits(v);
    }

    pub fn get_reg_hl(&self) -> u16 {
        to_u16(self.h, self.l)
    }

    pub fn set_reg_hl(&mut self, v: u16) {
        self.h = high_bits(v);
        self.l = low_bits(v);
    }
}

fn to_u16(h: u8, l: u8) -> u16 {
    (h as u16) << 8 | l as u16
}

fn high_bits(x: u16) -> u8 {
    (x >> 8) as u8
}

fn low_bits(x: u16) -> u8 {
    (x & 0xFF) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_registers() -> Registers {
        Registers::new()
    }

    #[test]
    fn test_get_reg_af() {
        let mut regs = mock_registers();
        regs.a = 0xF0;
        regs.z_flag = true;
        regs.n_flag = false;
        regs.h_flag = true;
        regs.c_flag = false;
        assert_eq!(0b1111000010100000, regs.get_reg_af());
    }

    #[test]
    fn test_set_reg_af() {
        let mut regs = mock_registers();
        regs.set_reg_af(0b1111000010100000);
        assert_eq!(regs.a, 0xF0);
        assert_eq!(true, regs.z_flag);
        assert_eq!(false, regs.n_flag);
        assert_eq!(true, regs.h_flag);
        assert_eq!(false, regs.c_flag);
    }
}
