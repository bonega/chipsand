use std::fmt::Debug;

use crate::Interrupt;
use crate::mmu::MMU;
use crate::registers::{R16, R8, RegIO, Registers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LitU8;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LitU16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mem {
    R16(R16),
    FF00C,
    FF00U8,
    LitU16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cond {
    NoCond,
    NZ,
    Z,
    NC,
    C,
}

impl Cond {
    fn is_true(&self, cpu: &CPU) -> bool {
        match self {
            Cond::NoCond => true,
            Cond::NZ => !cpu.regs.z_flag,
            Cond::Z => cpu.regs.z_flag,
            Cond::NC => !cpu.regs.c_flag,
            Cond::C => cpu.regs.c_flag,
        }
    }
}

pub trait Operand {
    fn to_str(&self, cpu: &CPU) -> String;
}

impl Operand for R8 {
    fn to_str(&self, _cpu: &CPU) -> String {
        format!("{:?}", self)
    }
}

impl Operand for R16 {
    fn to_str(&self, _cpu: &CPU) -> String {
        format!("{:?}", self)
    }
}

impl Operand for Mem {
    fn to_str(&self, cpu: &CPU) -> String {
        match self {
            Mem::R16(rr) => format!("({:?})", rr),
            Mem::FF00C => "(FF00+C)".to_string(),
            Mem::FF00U8 => format!("(FF00+{:X})", cpu.mmu.read_word(cpu.regs.pc)),
            Mem::LitU16 => format!("(0x{:X})", cpu.mmu.read_dw(cpu.regs.pc)),
        }
    }
}

impl Operand for LitU8 {
    fn to_str(&self, cpu: &CPU) -> String {
        format!("0x{:X}", cpu.mmu.read_word(cpu.regs.pc))
    }
}

impl Operand for LitU16 {
    fn to_str(&self, cpu: &CPU) -> String {
        format!("0x{:X}", cpu.mmu.read_dw(cpu.regs.pc))
    }
}

pub trait Target<T>: Debug + Copy + Operand {
    fn write(self, cpu: &mut CPU, v: T);
}

pub trait Source<T>: Debug + Copy + Operand {
    fn read(self, cpu: &mut CPU) -> T;
}

impl Target<u8> for R8 {
    fn write(self, cpu: &mut CPU, v: u8) {
        cpu.regs.write(self, v);
    }
}

impl Source<u8> for R8 {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.regs.read(self)
    }
}

impl Source<u8> for LitU8 {
    fn read(self, cpu: &mut CPU) -> u8 {
        let v = cpu.next_word();
        v
    }
}

impl Target<u16> for R16 {
    fn write(self, cpu: &mut CPU, v: u16) {
        cpu.regs.write(self, v);
    }
}

impl Source<u16> for R16 {
    fn read(self, cpu: &mut CPU) -> u16 {
        let v = cpu.regs.read(self);
        v
    }
}

impl Source<u16> for LitU16 {
    fn read(self, cpu: &mut CPU) -> u16 {
        let h = cpu.next_word();
        let l = cpu.next_word();
        let v = to_u16(l, h);
        v
    }
}

fn get_adr(cpu: &mut CPU, mem: Mem) -> u16 {
    let adr = match mem {
        Mem::R16(rr) => cpu.regs.read(rr),
        Mem::LitU16 => cpu.next_dw(),
        Mem::FF00C => 0xFF00 + cpu.regs.c as u16,
        Mem::FF00U8 => 0xFF00 + cpu.next_word() as u16,
    };
    adr
}

impl Source<u8> for Mem {
    fn read(self, cpu: &mut CPU) -> u8 {
        let adr = get_adr(cpu, self);
        cpu.tick();
        cpu.read_word(adr)
    }
}

impl Source<u16> for Mem {
    fn read(self, cpu: &mut CPU) -> u16 {
        let adr = get_adr(cpu, self);
        cpu.tick();
        cpu.tick();
        cpu.read_dw(adr)
    }
}

impl Target<u8> for Mem {
    fn write(self, cpu: &mut CPU, v: u8) {
        let adr = get_adr(cpu, self);
        cpu.tick();
        cpu.write_word(adr, v);
    }
}

impl Target<u16> for Mem {
    fn write(self, cpu: &mut CPU, v: u16) {
        let adr = get_adr(cpu, self);
        cpu.tick();
        cpu.tick();
        cpu.write_dw(adr, v);
    }
}

pub struct CPU {
    pub mmu: MMU,
    ime: bool, //Interrupt master enable flag
    pub timing: u8,
    pub regs: Registers,
    pub cycles: u64,
    is_halted: bool,
}

// Todo: Use union and unsafe for u16/u8u8 registers?
impl CPU {
    pub fn new(mmu: MMU) -> Self {
        CPU {
            mmu,
            ime: false,
            timing: 0,
            regs: Registers::new(),
            cycles: 0,
            is_halted: false,
        }
    }

    pub fn reset(&mut self) {
        self.regs.set_reg_af(0x01B0);
        self.regs.set_reg_bc(0x0013);
        self.regs.set_reg_de(0x00D8);
        self.regs.set_reg_hl(0x014D);
        self.regs.sp = 0xFFFE;
        self.mmu.write_word(0xFF05, 0);
        self.mmu.write_word(0xFF06, 0);
        self.mmu.write_word(0xFF07, 0);
        self.mmu.write_word(0xFF10, 0x80);
        self.mmu.write_word(0xFF11, 0xBF);
        self.mmu.write_word(0xFF12, 0xF3);
        self.mmu.write_word(0xFF14, 0xBF);
        self.mmu.write_word(0xFF16, 0x3F);
        self.mmu.write_word(0xFF17, 0);
        self.mmu.write_word(0xFF19, 0xBF);
        self.mmu.write_word(0xFF1A, 0x7F);
        self.mmu.write_word(0xFF1B, 0xFF);
        self.mmu.write_word(0xFF1C, 0x9F);
        self.mmu.write_word(0xFF1E, 0xBF);
        self.mmu.write_word(0xFF20, 0xFF);
        self.mmu.write_word(0xFF21, 0);
        self.mmu.write_word(0xFF22, 0);
        self.mmu.write_word(0xFF23, 0xBF);
        self.mmu.write_word(0xFF24, 0x77);
        self.mmu.write_word(0xFF25, 0xF3);
        self.mmu.write_word(0xFF26, 0xF1);
        self.mmu.write_word(0xFF40, 0x91); // LCDC
        self.mmu.write_word(0xFF42, 0); // SCY
        self.mmu.write_word(0xFF43, 0); // SCX
        self.mmu.write_word(0xFF45, 0); // LYC
        self.mmu.write_word(0xFF47, 0xFC); // BGP
        self.mmu.write_word(0xFF48, 0xFF); // OBP0
        self.mmu.write_word(0xFF49, 0xFF); // OBP1
        self.mmu.write_word(0xFF4A, 0); // WY
        self.mmu.write_word(0xFF4B, 0); // WX
        self.mmu.write_word(0xFFFF, 0); // IE
    }

    pub fn ld<T>(&mut self, target: impl Target<T>, source: impl Source<T>) {
        #[cfg(feature = "op-debug")]
        println!("LD {}, {}", target.to_str(self), source.to_str(self));
        let v = source.read(self);
        target.write(self, v);
    }

    fn ld_sp_hl(&mut self, target: R16, source: R16) {
        #[cfg(feature = "op-debug")]
        println!("LD {}, {}", target.to_str(self), source.to_str(self));
        let v = source.read(self);
        self.tick();
        target.write(self, v);
    }

    fn ld_hl_sp_i8(&mut self) {
        let v = self.next_word() as i8 as u16;
        #[cfg(feature = "op-debug")]
        println!("LD HL, SP+0x{:X}", v);
        let h = ((self.regs.sp & 0x000F) + (v & 0x000F)) > 0x000F;
        let carry = (self.regs.sp & 0x00FF) + (v & 0x00FF) > 0x00FF;
        self.regs.set_reg_hl(self.regs.sp.wrapping_add(v));
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = h;
        self.regs.c_flag = carry;
        self.tick();
    }

    pub fn ldi<T>(&mut self, target: impl Target<T>, source: impl Source<T>) {
        #[cfg(feature = "op-debug")]
        println!("LDI {}, {}", target.to_str(self), source.to_str(self));
        let v = source.read(self);
        target.write(self, v);
        self.regs.set_reg_hl(self.regs.get_reg_hl().wrapping_add(1))
    }

    pub fn ldd<T>(&mut self, target: impl Target<T>, source: impl Source<T>) {
        #[cfg(feature = "op-debug")]
        println!("LDD {}, {}", target.to_str(self), source.to_str(self));
        let v = source.read(self);
        target.write(self, v);
        self.regs.set_reg_hl(self.regs.get_reg_hl().wrapping_sub(1))
    }

    fn inc_u8(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("INC {}", target.to_str(self));
        let v = target.read(self);
        self.regs.z_flag = v == 0xFF;
        self.regs.n_flag = false;
        self.regs.h_flag = ((v & 0xF) + 1) & 0x10 != 0;
        target.write(self, v.wrapping_add(1));
    }

    fn inc_u16(&mut self, target: impl Source<u16> + Target<u16>) {
        #[cfg(feature = "op-debug")]
        println!("INC {}", target.to_str(self));
        let v = target.read(self);
        self.tick();
        target.write(self, v.wrapping_add(1));
    }

    fn dec_u8(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("DEC {}", target.to_str(self));
        let v = target.read(self);
        self.regs.z_flag = 1 == v;
        self.regs.n_flag = true;
        self.regs.h_flag = (v & 0xF) == 0;
        target.write(self, v.wrapping_sub(1));
    }

    fn dec_u16(&mut self, target: impl Source<u16> + Target<u16>) {
        #[cfg(feature = "op-debug")]
        println!("DEC {}", target.to_str(self));
        let v = target.read(self).wrapping_sub(1);
        self.tick();
        target.write(self, v);
    }

    fn add_a_r(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("ADD A, {}", source.to_str(self));
        let v = source.read(self);
        let (sum, carry) = self.regs.a.overflowing_add(v);
        self.regs.z_flag = sum == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = half_carry(self.regs.a, v);
        self.regs.c_flag = carry;
        self.regs.a = sum;
    }

    fn add_hl_rr(&mut self, source: R16) {
        #[cfg(feature = "op-debug")]
        println!("ADD HL, {}", source.to_str(self));
        let v = source.read(self);
        let hl = self.regs.get_reg_hl();
        let (sum, carry) = hl.overflowing_add(v);
        self.regs.n_flag = false;
        self.regs.h_flag = half_carry_u16(hl, v);
        self.regs.c_flag = carry as bool;
        self.regs.set_reg_hl(sum);
        self.tick();
    }

    fn add_sp(&mut self) {
        let v = self.next_word() as i8 as u16;
        #[cfg(feature = "op-debug")]
        println!("ADD SP, 0x{:X}", v);
        let h = ((self.regs.sp & 0x000F) + (v & 0x000F)) > 0x000F;
        let carry = (self.regs.sp & 0x00FF) + (v & 0x00FF) > 0x00FF;
        self.regs.sp = self.regs.sp.wrapping_add(v);
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = h;
        self.regs.c_flag = carry;
        self.tick();
        self.tick();
    }

    fn adc(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("ADC A, {}", source.to_str(self));
        let a = self.regs.a;
        let v = source.read(self);
        let c = self.regs.c_flag as u8;
        let sum = a.wrapping_add(v).wrapping_add(c);
        self.regs.z_flag = sum == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = (a & 0xF) + (v & 0xF) + c > 0xF;
        self.regs.c_flag = (a as u16) + (v as u16) + (c as u16) > 0xFF;
        self.regs.a = sum;
    }

    fn sub(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SUB A, {}", source.to_str(self));
        let a = self.regs.a;
        let v = source.read(self);
        let (sum, carry) = self.regs.a.overflowing_sub(v);
        self.regs.z_flag = sum == 0;
        self.regs.n_flag = true;
        self.regs.h_flag = (a & 0xF) < (v & 0xF);
        self.regs.c_flag = carry;
        self.regs.a = sum;
    }

    fn cp(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("CP A, {}", source.to_str(self));
        let a = self.regs.a;
        let v = source.read(self);
        let (sum, carry) = self.regs.a.overflowing_sub(v);
        self.regs.z_flag = sum == 0;
        self.regs.n_flag = true;
        self.regs.h_flag = (a & 0xF) < (v & 0xF);
        self.regs.c_flag = carry;
    }

    fn sbc(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SBC A, {}", source.to_str(self));
        let a = self.regs.a;
        let v = source.read(self);
        let sum = a.wrapping_sub(v).wrapping_sub(self.regs.c_flag as u8);
        let carry = (a as u16) < (v as u16) + self.regs.c_flag as u16;
        self.regs.z_flag = sum == 0;
        self.regs.n_flag = true;
        self.regs.h_flag = (a & 0xF) < (v & 0xF) + self.regs.c_flag as u8;
        self.regs.c_flag = carry;
        self.regs.a = sum;
    }

    fn and(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("AND A, {}", source.to_str(self));
        let v = source.read(self);
        self.regs.a &= v;
        self.regs.z_flag = self.regs.a == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = true;
        self.regs.c_flag = false;
    }

    fn xor(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("XOR A, {}", source.to_str(self));
        let v = source.read(self);
        self.regs.a ^= v;
        self.regs.z_flag = self.regs.a == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = false;
        self.regs.a;
    }

    fn or(&mut self, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("OR A, {}", source.to_str(self));
        let v = source.read(self);
        self.regs.a |= v;
        self.regs.z_flag = self.regs.a == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = false;
    }

    fn rla(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("RLA");
        let a = self.regs.a;
        let rot_bit = a & 0x80;
        self.regs.a = ((a << 1) | self.regs.c_flag as u8) & 0xFF;
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rlca(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("RLCA");
        let a = self.regs.a;
        let rot_bit = a >> 7;
        self.regs.a = ((a << 1) | rot_bit) & 0xFF;
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rl(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("RL {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r & 0x80;
        let v = ((r << 1) | self.regs.c_flag as u8) & 0xFF;
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rlc(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("RLC {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r >> 7;
        let v = ((r << 1) | rot_bit) & 0xFF;
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rr(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("RR {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r & 1;
        let v = (r >> 1) | ((self.regs.c_flag as u8) << 7);
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rrc(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("RRC {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r & 1;
        let v = (r >> 1) | (rot_bit << 7);
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn sla(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SLA {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r >> 7;
        let v = (r << 1) & 0xFF;
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn sra(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SRA {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r & 0x01;
        let v = (r >> 1) | (r & 0x80);
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn srl(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SRL {}", target.to_str(self));
        let r = target.read(self);
        let rot_bit = r & 0x01;
        let v = r >> 1;
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rra(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("RRA");
        let a = self.regs.a;
        let rot_bit = a & 1;
        self.regs.a = (a >> 1) | ((self.regs.c_flag as u8) << 7);
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn rrca(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("RRCA");
        let a = self.regs.a;
        let rot_bit = a & 1;
        self.regs.a = (a >> 1) | (rot_bit << 7);
        self.regs.z_flag = false;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = rot_bit != 0;
    }

    fn jr(&mut self, cond: Cond) {
        let offset = self.next_word() as i8;
        #[cfg(feature = "op-debug")]
        println!("JR {:?}, 0x{:X}", cond, offset);
        if cond.is_true(self) {
            self.tick();
            self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
        }
    }

    fn jp_u16(&mut self, cond: Cond) {
        let adr = self.next_dw();
        #[cfg(feature = "op-debug")]
        println!("JP {:?}, 0x{:X}", cond, adr);
        if cond.is_true(self) {
            self.tick();
            self.regs.pc = adr;
        }
    }

    fn jp_hl(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("JP HL");
        self.regs.pc = self.regs.get_reg_hl();
    }

    fn ret(&mut self, cond: Cond) {
        #[cfg(feature = "op-debug")]
        println!("RET {:?}", cond);
        self.tick();
        if cond.is_true(self) {
            if cond != Cond::NoCond {
                self.tick();
            }
            let v = Mem::R16(R16::SP).read(self);
            self.regs.pc = v;
            self.regs.sp += 2;
        }
    }

    fn reti(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("RETI");
        self.ret(Cond::NoCond);
        self.ime = true;
    }

    fn call(&mut self, cond: Cond) {
        let adr = self.next_dw();
        #[cfg(feature = "op-debug")]
        println!("CALL {:?}, 0x{:X}", cond, adr);
        if cond.is_true(self) {
            self.push_u16(self.regs.pc);
            self.regs.pc = adr;
        }
    }

    fn rst(&mut self, adr: u8) {
        #[cfg(feature = "op-debug")]
        println!("RST 0x{:X}", adr);
        self.push_u16(self.regs.pc);
        self.regs.pc = adr as u16;
    }

    fn push_u16(&mut self, v: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.tick();
        Mem::R16(R16::SP).write(self, v);
    }

    fn push(&mut self, rr: R16) {
        #[cfg(feature = "op-debug")]
        println!("PUSH {:?}", rr);
        let v = rr.read(self);
        self.push_u16(v);
    }

    fn pop(&mut self, rr: R16) {
        #[cfg(feature = "op-debug")]
        println!("POP {:?}", rr);
        let v = Mem::R16(R16::SP).read(self);
        rr.write(self, v);
        self.regs.sp += 2;
    }

    fn daa(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("DAA");
        let mut a = self.regs.a;
        if self.regs.n_flag {
            if self.regs.c_flag {
                a = a.wrapping_sub(0x60);
            }
            if self.regs.h_flag {
                a = a.wrapping_sub(0x6);
            }
        } else {
            if self.regs.c_flag || a > 0x99 {
                a = a.wrapping_add(0x60);
                self.regs.c_flag = true;
            }
            if self.regs.h_flag || (a & 0x0F) > 0x09 {
                a = a.wrapping_add(0x6);
            }
        }
        self.regs.a = a;
        self.regs.z_flag = self.regs.a == 0;
        self.regs.h_flag = false;
    }

    fn cpl(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("CPL");
        self.regs.n_flag = true;
        self.regs.h_flag = true;
        self.regs.a ^= 0xFF;
    }

    fn scf(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("SCF");
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = true;
    }

    fn ccf(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("CCF");
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = !self.regs.c_flag;
    }

    fn halt(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("HALT");
        if self.mmu.get_interrupts() == 0 {
            self.is_halted = true;
            self.regs.pc = self.regs.pc.wrapping_sub(1);
        } else {
            self.is_halted = false;
        }
    }

    pub fn read_word(&self, adr: u16) -> u8 {
        self.mmu.read_word(adr)
    }

    pub fn read_dw(&self, adr: u16) -> u16 {
        self.mmu.read_dw(adr)
    }

    pub fn write_word(&mut self, adr: u16, v: u8) {
        self.mmu.write_word(adr, v);
    }

    pub fn write_dw(&mut self, adr: u16, v: u16) {
        self.mmu.write_dw(adr, v);
    }

    pub fn next_word(&mut self) -> u8 {
        let res = self.mmu.read_word(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.tick();
        res
    }

    fn next_dw(&mut self) -> u16 {
        let pc = self.regs.pc;
        let res = self.mmu.read_dw(pc);
        self.regs.pc += 2;
        self.tick();
        self.tick();
        res
    }

    fn stop(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("STOP");
        unimplemented!();
        //self.next_word();
    }

    fn ei(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("EI");
        self.ime = true;
    }

    fn di(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("DI");
        self.ime = false;
    }

    fn swap(&mut self, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SWAP {}", target.to_str(self));
        let mut v = target.read(self);
        let h = v >> 4;
        let l = v & 0xF;
        v = h | (l << 4);
        target.write(self, v);
        self.regs.z_flag = v == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = false;
        self.regs.c_flag = false
    }

    fn bit(&mut self, n: u8, source: impl Source<u8>) {
        #[cfg(feature = "op-debug")]
        println!("BIT {}, {}", n, source.to_str(self));
        let v = source.read(self);
        self.regs.z_flag = ((v >> n) & 1) == 0;
        self.regs.n_flag = false;
        self.regs.h_flag = true;
    }

    fn res(&mut self, n: u8, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("RES {}, {}", n, target.to_str(self));
        let v = target.read(self);
        let res_mask = !(1 << n);
        target.write(self, v & res_mask);
    }

    fn set(&mut self, n: u8, target: impl Source<u8> + Target<u8>) {
        #[cfg(feature = "op-debug")]
        println!("SET {}, {}", n, target.to_str(self));
        let v = target.read(self);
        let set_mask = 1 << n;
        target.write(self, v | set_mask);
    }

    fn nop(&mut self) {
        #[cfg(feature = "op-debug")]
        println!("NOP");
    }

    pub fn execute(&mut self, opcode: u8) {
        //        #[cfg(feature = "op-debug")]
        //        println!("Opcode 0x{:X}",opcode);
        use super::registers::R16::*;
        use super::registers::R8::*;
        match opcode {
            0x00 => self.nop(),
            0x01 => self.ld(BC, LitU16),
            0x02 => self.ld(Mem::R16(BC), A),
            0x03 => self.inc_u16(BC),
            0x04 => self.inc_u8(B),
            0x05 => self.dec_u8(B),
            0x06 => self.ld(B, LitU8),
            0x07 => self.rlca(),
            0x08 => self.ld(Mem::LitU16, SP),
            0x09 => self.add_hl_rr(BC),
            0x0A => self.ld(A, Mem::R16(BC)),
            0x0B => self.dec_u16(BC),
            0x0C => self.inc_u8(C),
            0x0D => self.dec_u8(C),
            0x0E => self.ld(C, LitU8),
            0x0F => self.rrca(),
            0x10 => self.stop(),
            0x11 => self.ld(DE, LitU16),
            0x12 => self.ld(Mem::R16(DE), A),
            0x13 => self.inc_u16(DE),
            0x14 => self.inc_u8(D),
            0x15 => self.dec_u8(D),
            0x16 => self.ld(D, LitU8),
            0x17 => self.rla(),
            0x18 => self.jr(Cond::NoCond),
            0x19 => self.add_hl_rr(DE),
            0x1A => self.ld(A, Mem::R16(DE)),
            0x1B => self.dec_u16(DE),
            0x1C => self.inc_u8(E),
            0x1D => self.dec_u8(E),
            0x1E => self.ld(E, LitU8),
            0x1F => self.rra(),
            0x20 => self.jr(Cond::NZ),
            0x21 => self.ld(HL, LitU16),
            0x22 => self.ldi(Mem::R16(HL), A),
            0x23 => self.inc_u16(HL),
            0x24 => self.inc_u8(H),
            0x25 => self.dec_u8(H),
            0x26 => self.ld(H, LitU8),
            0x27 => self.daa(),
            0x28 => self.jr(Cond::Z),
            0x29 => self.add_hl_rr(HL),
            0x2A => self.ldi(A, Mem::R16(HL)),
            0x2B => self.dec_u16(HL),
            0x2C => self.inc_u8(L),
            0x2D => self.dec_u8(L),
            0x2E => self.ld(L, LitU8),
            0x2F => self.cpl(),
            0x30 => self.jr(Cond::NC),
            0x31 => self.ld(SP, LitU16),
            0x32 => self.ldd(Mem::R16(HL), A),
            0x33 => self.inc_u16(SP),
            0x34 => self.inc_u8(Mem::R16(HL)),
            0x35 => self.dec_u8(Mem::R16(HL)),
            0x36 => self.ld(Mem::R16(HL), LitU8),
            0x37 => self.scf(),
            0x38 => self.jr(Cond::C),
            0x39 => self.add_hl_rr(SP),
            0x3A => self.ldd(A, Mem::R16(HL)),
            0x3B => self.dec_u16(SP),
            0x3C => self.inc_u8(A),
            0x3D => self.dec_u8(A),
            0x3E => self.ld(A, LitU8),
            0x3F => self.ccf(),
            0x40 => self.ld(B, B),
            0x41 => self.ld(B, C),
            0x42 => self.ld(B, D),
            0x43 => self.ld(B, E),
            0x44 => self.ld(B, H),
            0x45 => self.ld(B, L),
            0x46 => self.ld(B, Mem::R16(HL)),
            0x47 => self.ld(B, A),
            0x48 => self.ld(C, B),
            0x49 => self.ld(C, C),
            0x4A => self.ld(C, D),
            0x4B => self.ld(C, E),
            0x4C => self.ld(C, H),
            0x4D => self.ld(C, L),
            0x4E => self.ld(C, Mem::R16(HL)),
            0x4F => self.ld(C, A),
            0x50 => self.ld(D, B),
            0x51 => self.ld(D, C),
            0x52 => self.ld(D, D),
            0x53 => self.ld(D, E),
            0x54 => self.ld(D, H),
            0x55 => self.ld(D, L),
            0x56 => self.ld(D, Mem::R16(HL)),
            0x57 => self.ld(D, A),
            0x58 => self.ld(E, B),
            0x59 => self.ld(E, C),
            0x5A => self.ld(E, D),
            0x5B => self.ld(E, E),
            0x5C => self.ld(E, H),
            0x5D => self.ld(E, L),
            0x5E => self.ld(E, Mem::R16(HL)),
            0x5F => self.ld(E, A),
            0x60 => self.ld(H, B),
            0x61 => self.ld(H, C),
            0x62 => self.ld(H, D),
            0x63 => self.ld(H, E),
            0x64 => self.ld(H, H),
            0x65 => self.ld(H, L),
            0x66 => self.ld(H, Mem::R16(HL)),
            0x67 => self.ld(H, A),
            0x68 => self.ld(L, B),
            0x69 => self.ld(L, C),
            0x6A => self.ld(L, D),
            0x6B => self.ld(L, E),
            0x6C => self.ld(L, H),
            0x6D => self.ld(L, L),
            0x6E => self.ld(L, Mem::R16(HL)),
            0x6F => self.ld(L, A),
            0x70 => self.ld(Mem::R16(HL), B),
            0x71 => self.ld(Mem::R16(HL), C),
            0x72 => self.ld(Mem::R16(HL), D),
            0x73 => self.ld(Mem::R16(HL), E),
            0x74 => self.ld(Mem::R16(HL), H),
            0x75 => self.ld(Mem::R16(HL), L),
            0x76 => self.halt(),
            0x77 => self.ld(Mem::R16(HL), A),
            0x78 => self.ld(A, B),
            0x79 => self.ld(A, C),
            0x7A => self.ld(A, D),
            0x7B => self.ld(A, E),
            0x7C => self.ld(A, H),
            0x7D => self.ld(A, L),
            0x7E => self.ld(A, Mem::R16(HL)),
            0x7F => self.ld(A, A),
            0x80 => self.add_a_r(B),
            0x81 => self.add_a_r(C),
            0x82 => self.add_a_r(D),
            0x83 => self.add_a_r(E),
            0x84 => self.add_a_r(H),
            0x85 => self.add_a_r(L),
            0x86 => self.add_a_r(Mem::R16(HL)),
            0x87 => self.add_a_r(A),
            0x88 => self.adc(B),
            0x89 => self.adc(C),
            0x8A => self.adc(D),
            0x8B => self.adc(E),
            0x8C => self.adc(H),
            0x8D => self.adc(L),
            0x8E => self.adc(Mem::R16(HL)),
            0x8F => self.adc(A),
            0x90 => self.sub(B),
            0x91 => self.sub(C),
            0x92 => self.sub(D),
            0x93 => self.sub(E),
            0x94 => self.sub(H),
            0x95 => self.sub(L),
            0x96 => self.sub(Mem::R16(HL)),
            0x97 => self.sub(A),
            0x98 => self.sbc(B),
            0x99 => self.sbc(C),
            0x9A => self.sbc(D),
            0x9B => self.sbc(E),
            0x9C => self.sbc(H),
            0x9D => self.sbc(L),
            0x9E => self.sbc(Mem::R16(HL)),
            0x9F => self.sbc(A),
            0xA0 => self.and(B),
            0xA1 => self.and(C),
            0xA2 => self.and(D),
            0xA3 => self.and(E),
            0xA4 => self.and(H),
            0xA5 => self.and(L),
            0xA6 => self.and(Mem::R16(HL)),
            0xA7 => self.and(A),
            0xA8 => self.xor(B),
            0xA9 => self.xor(C),
            0xAA => self.xor(D),
            0xAB => self.xor(E),
            0xAC => self.xor(H),
            0xAD => self.xor(L),
            0xAE => self.xor(Mem::R16(HL)),
            0xAF => self.xor(A),
            0xB0 => self.or(B),
            0xB1 => self.or(C),
            0xB2 => self.or(D),
            0xB3 => self.or(E),
            0xB4 => self.or(H),
            0xB5 => self.or(L),
            0xB6 => self.or(Mem::R16(HL)),
            0xB7 => self.or(A),
            0xB8 => self.cp(B),
            0xB9 => self.cp(C),
            0xBA => self.cp(D),
            0xBB => self.cp(E),
            0xBC => self.cp(H),
            0xBD => self.cp(L),
            0xBE => self.cp(Mem::R16(HL)),
            0xBF => self.cp(A),
            0xC0 => self.ret(Cond::NZ),
            0xC1 => self.pop(BC),
            0xC2 => self.jp_u16(Cond::NZ),
            0xC3 => self.jp_u16(Cond::NoCond),
            0xC4 => self.call(Cond::NZ),
            0xC5 => self.push(BC),
            0xC6 => self.add_a_r(LitU8),
            0xC7 => self.rst(0x00),
            0xC8 => self.ret(Cond::Z),
            0xC9 => self.ret(Cond::NoCond),
            0xCA => self.jp_u16(Cond::Z),
            0xCB => {
                let cb = self.next_word();
                self.execute_cb(cb)
            }
            0xCC => self.call(Cond::Z),
            0xCD => self.call(Cond::NoCond),
            0xCE => self.adc(LitU8),
            0xCF => self.rst(0x08),
            0xD0 => self.ret(Cond::NC),
            0xD1 => self.pop(DE),
            0xD2 => self.jp_u16(Cond::NC),
            0xD4 => self.call(Cond::NC),
            0xD5 => self.push(DE),
            0xD6 => self.sub(LitU8),
            0xD7 => self.rst(0x10),
            0xD8 => self.ret(Cond::C),
            0xD9 => self.reti(),
            0xDA => self.jp_u16(Cond::C),
            0xDC => self.call(Cond::C),
            0xDE => self.sbc(LitU8),
            0xDF => self.rst(0x18),
            0xE0 => self.ld(Mem::FF00U8, A),
            0xE1 => self.pop(HL),
            0xE2 => self.ld(Mem::FF00C, A),
            0xE5 => self.push(HL),
            0xE6 => self.and(LitU8),
            0xE7 => self.rst(0x20),
            0xE8 => self.add_sp(),
            0xE9 => self.jp_hl(),
            0xEA => self.ld(Mem::LitU16, A),
            0xEE => self.xor(LitU8),
            0xEF => self.rst(0x28),
            0xF0 => self.ld(A, Mem::FF00U8),
            0xF1 => self.pop(AF),
            0xF2 => self.ld(A, Mem::FF00C),
            0xF3 => self.di(),
            0xF5 => self.push(AF),
            0xF6 => self.or(LitU8),
            0xF7 => self.rst(0x30),
            0xF8 => self.ld_hl_sp_i8(),
            0xF9 => self.ld_sp_hl(SP, HL),
            0xFA => self.ld(A, Mem::LitU16),
            0xFB => self.ei(),
            0xFE => self.cp(LitU8),
            0xFF => self.rst(0x38),
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB..=0xED | 0xF4 | 0xFC | 0xFD => {
                panic!("Illegal instruction {}", opcode)
            }
        }
    }

    fn execute_cb(&mut self, opcode: u8) {
        use super::registers::R16::*;
        use super::registers::R8::*;
        match opcode {
            0x00 => self.rlc(B),
            0x01 => self.rlc(C),
            0x02 => self.rlc(D),
            0x03 => self.rlc(E),
            0x04 => self.rlc(H),
            0x05 => self.rlc(L),
            0x06 => self.rlc(Mem::R16(HL)),
            0x07 => self.rlc(A),
            0x08 => self.rrc(B),
            0x09 => self.rrc(C),
            0x0A => self.rrc(D),
            0x0B => self.rrc(E),
            0x0C => self.rrc(H),
            0x0D => self.rrc(L),
            0x0E => self.rrc(Mem::R16(HL)),
            0x0F => self.rrc(A),
            0x10 => self.rl(B),
            0x11 => self.rl(C),
            0x12 => self.rl(D),
            0x13 => self.rl(E),
            0x14 => self.rl(H),
            0x15 => self.rl(L),
            0x16 => self.rl(Mem::R16(HL)),
            0x17 => self.rl(A),
            0x18 => self.rr(B),
            0x19 => self.rr(C),
            0x1A => self.rr(D),
            0x1B => self.rr(E),
            0x1C => self.rr(H),
            0x1D => self.rr(L),
            0x1E => self.rr(Mem::R16(HL)),
            0x1F => self.rr(A),
            0x20 => self.sla(B),
            0x21 => self.sla(C),
            0x22 => self.sla(D),
            0x23 => self.sla(E),
            0x24 => self.sla(H),
            0x25 => self.sla(L),
            0x26 => self.sla(Mem::R16(HL)),
            0x27 => self.sla(A),
            0x28 => self.sra(B),
            0x29 => self.sra(C),
            0x2A => self.sra(D),
            0x2B => self.sra(E),
            0x2C => self.sra(H),
            0x2D => self.sra(L),
            0x2E => self.sra(Mem::R16(HL)),
            0x2F => self.sra(A),
            0x30 => self.swap(B),
            0x31 => self.swap(C),
            0x32 => self.swap(D),
            0x33 => self.swap(E),
            0x34 => self.swap(H),
            0x35 => self.swap(L),
            0x36 => self.swap(Mem::R16(HL)),
            0x37 => self.swap(A),
            0x38 => self.srl(B),
            0x39 => self.srl(C),
            0x3A => self.srl(D),
            0x3B => self.srl(E),
            0x3C => self.srl(H),
            0x3D => self.srl(L),
            0x3E => self.srl(Mem::R16(HL)),
            0x3F => self.srl(A),
            0x40 => self.bit(0, B),
            0x41 => self.bit(0, C),
            0x42 => self.bit(0, D),
            0x43 => self.bit(0, E),
            0x44 => self.bit(0, H),
            0x45 => self.bit(0, L),
            0x46 => self.bit(0, Mem::R16(HL)),
            0x47 => self.bit(0, A),
            0x48 => self.bit(1, B),
            0x49 => self.bit(1, C),
            0x4A => self.bit(1, D),
            0x4B => self.bit(1, E),
            0x4C => self.bit(1, H),
            0x4D => self.bit(1, L),
            0x4E => self.bit(1, Mem::R16(HL)),
            0x4F => self.bit(1, A),
            0x50 => self.bit(2, B),
            0x51 => self.bit(2, C),
            0x52 => self.bit(2, D),
            0x53 => self.bit(2, E),
            0x54 => self.bit(2, H),
            0x55 => self.bit(2, L),
            0x56 => self.bit(2, Mem::R16(HL)),
            0x57 => self.bit(2, A),
            0x58 => self.bit(3, B),
            0x59 => self.bit(3, C),
            0x5A => self.bit(3, D),
            0x5B => self.bit(3, E),
            0x5C => self.bit(3, H),
            0x5D => self.bit(3, L),
            0x5E => self.bit(3, Mem::R16(HL)),
            0x5F => self.bit(3, A),
            0x60 => self.bit(4, B),
            0x61 => self.bit(4, C),
            0x62 => self.bit(4, D),
            0x63 => self.bit(4, E),
            0x64 => self.bit(4, H),
            0x65 => self.bit(4, L),
            0x66 => self.bit(4, Mem::R16(HL)),
            0x67 => self.bit(4, A),
            0x68 => self.bit(5, B),
            0x69 => self.bit(5, C),
            0x6A => self.bit(5, D),
            0x6B => self.bit(5, E),
            0x6C => self.bit(5, H),
            0x6D => self.bit(5, L),
            0x6E => self.bit(5, Mem::R16(HL)),
            0x6F => self.bit(5, A),
            0x70 => self.bit(6, B),
            0x71 => self.bit(6, C),
            0x72 => self.bit(6, D),
            0x73 => self.bit(6, E),
            0x74 => self.bit(6, H),
            0x75 => self.bit(6, L),
            0x76 => self.bit(6, Mem::R16(HL)),
            0x77 => self.bit(6, A),
            0x78 => self.bit(7, B),
            0x79 => self.bit(7, C),
            0x7A => self.bit(7, D),
            0x7B => self.bit(7, E),
            0x7C => self.bit(7, H),
            0x7D => self.bit(7, L),
            0x7E => self.bit(7, Mem::R16(HL)),
            0x7F => self.bit(7, A),
            0x80 => self.res(0, B),
            0x81 => self.res(0, C),
            0x82 => self.res(0, D),
            0x83 => self.res(0, E),
            0x84 => self.res(0, H),
            0x85 => self.res(0, L),
            0x86 => self.res(0, Mem::R16(HL)),
            0x87 => self.res(0, A),
            0x88 => self.res(1, B),
            0x89 => self.res(1, C),
            0x8A => self.res(1, D),
            0x8B => self.res(1, E),
            0x8C => self.res(1, H),
            0x8D => self.res(1, L),
            0x8E => self.res(1, Mem::R16(HL)),
            0x8F => self.res(1, A),
            0x90 => self.res(2, B),
            0x91 => self.res(2, C),
            0x92 => self.res(2, D),
            0x93 => self.res(2, E),
            0x94 => self.res(2, H),
            0x95 => self.res(2, L),
            0x96 => self.res(2, Mem::R16(HL)),
            0x97 => self.res(2, A),
            0x98 => self.res(3, B),
            0x99 => self.res(3, C),
            0x9A => self.res(3, D),
            0x9B => self.res(3, E),
            0x9C => self.res(3, H),
            0x9D => self.res(3, L),
            0x9E => self.res(3, Mem::R16(HL)),
            0x9F => self.res(3, A),
            0xA0 => self.res(4, B),
            0xA1 => self.res(4, C),
            0xA2 => self.res(4, D),
            0xA3 => self.res(4, E),
            0xA4 => self.res(4, H),
            0xA5 => self.res(4, L),
            0xA6 => self.res(4, Mem::R16(HL)),
            0xA7 => self.res(4, A),
            0xA8 => self.res(5, B),
            0xA9 => self.res(5, C),
            0xAA => self.res(5, D),
            0xAB => self.res(5, E),
            0xAC => self.res(5, H),
            0xAD => self.res(5, L),
            0xAE => self.res(5, Mem::R16(HL)),
            0xAF => self.res(5, A),
            0xB0 => self.res(6, B),
            0xB1 => self.res(6, C),
            0xB2 => self.res(6, D),
            0xB3 => self.res(6, E),
            0xB4 => self.res(6, H),
            0xB5 => self.res(6, L),
            0xB6 => self.res(6, Mem::R16(HL)),
            0xB7 => self.res(6, A),
            0xB8 => self.res(7, B),
            0xB9 => self.res(7, C),
            0xBA => self.res(7, D),
            0xBB => self.res(7, E),
            0xBC => self.res(7, H),
            0xBD => self.res(7, L),
            0xBE => self.res(7, Mem::R16(HL)),
            0xBF => self.res(7, A),
            0xC0 => self.set(0, B),
            0xC1 => self.set(0, C),
            0xC2 => self.set(0, D),
            0xC3 => self.set(0, E),
            0xC4 => self.set(0, H),
            0xC5 => self.set(0, L),
            0xC6 => self.set(0, Mem::R16(HL)),
            0xC7 => self.set(0, A),
            0xC8 => self.set(1, B),
            0xC9 => self.set(1, C),
            0xCA => self.set(1, D),
            0xCB => self.set(1, E),
            0xCC => self.set(1, H),
            0xCD => self.set(1, L),
            0xCE => self.set(1, Mem::R16(HL)),
            0xCF => self.set(1, A),
            0xD0 => self.set(2, B),
            0xD1 => self.set(2, C),
            0xD2 => self.set(2, D),
            0xD3 => self.set(2, E),
            0xD4 => self.set(2, H),
            0xD5 => self.set(2, L),
            0xD6 => self.set(2, Mem::R16(HL)),
            0xD7 => self.set(2, A),
            0xD8 => self.set(3, B),
            0xD9 => self.set(3, C),
            0xDA => self.set(3, D),
            0xDB => self.set(3, E),
            0xDC => self.set(3, H),
            0xDD => self.set(3, L),
            0xDE => self.set(3, Mem::R16(HL)),
            0xDF => self.set(3, A),
            0xE0 => self.set(4, B),
            0xE1 => self.set(4, C),
            0xE2 => self.set(4, D),
            0xE3 => self.set(4, E),
            0xE4 => self.set(4, H),
            0xE5 => self.set(4, L),
            0xE6 => self.set(4, Mem::R16(HL)),
            0xE7 => self.set(4, A),
            0xE8 => self.set(5, B),
            0xE9 => self.set(5, C),
            0xEA => self.set(5, D),
            0xEB => self.set(5, E),
            0xEC => self.set(5, H),
            0xED => self.set(5, L),
            0xEE => self.set(5, Mem::R16(HL)),
            0xEF => self.set(5, A),
            0xF0 => self.set(6, B),
            0xF1 => self.set(6, C),
            0xF2 => self.set(6, D),
            0xF3 => self.set(6, E),
            0xF4 => self.set(6, H),
            0xF5 => self.set(6, L),
            0xF6 => self.set(6, Mem::R16(HL)),
            0xF7 => self.set(6, A),
            0xF8 => self.set(7, B),
            0xF9 => self.set(7, C),
            0xFA => self.set(7, D),
            0xFB => self.set(7, E),
            0xFC => self.set(7, H),
            0xFD => self.set(7, L),
            0xFE => self.set(7, Mem::R16(HL)),
            0xFF => self.set(7, A),
        }
    }

    fn interrupt(&mut self, v: u8) {
        #[cfg(feature = "op-debug")]
        println!("INT 0x{:X}", v);
        //        println!("INT 0x{:X}", v);
        self.tick();
        self.tick();
        self.push_u16(self.regs.pc);
        self.regs.pc = v as u16;
    }

    pub fn cycle(&mut self) {
        self.timing = 0;
        let interrupts = self.mmu.interrupt_flags & self.mmu.interrupt_enable;
        if self.ime {
            if interrupts & Interrupt::VBLANK as u8 != 0 {
                self.mmu.interrupt_flags ^= Interrupt::VBLANK as u8;
                self.ime = false;
                self.interrupt(0x40);
            } else if interrupts & Interrupt::LCDStat as u8 != 0 {
                self.mmu.interrupt_flags ^= Interrupt::LCDStat as u8;
                self.ime = false;
                println!("interrupt 48");
                self.interrupt(0x48);
            } else if interrupts & Interrupt::TIMER as u8 != 0 {
                self.mmu.interrupt_flags ^= Interrupt::TIMER as u8;
                println!("reg c 0x{:X}", self.regs.c);
                self.ime = false;
                self.interrupt(0x50);
            } else if interrupts & Interrupt::SERIAL as u8 != 0 {
                self.mmu.interrupt_flags ^= Interrupt::SERIAL as u8;
                self.ime = false;
                self.interrupt(0x58);
            }
        }
        let opcode = self.next_word();
        self.execute(opcode);
    }

    pub fn tick(&mut self) {
        self.cycles += 4;
        self.mmu.tick();
    }
}

fn half_carry(a: u8, b: u8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) != 0
}

fn half_carry_u16(a: u16, b: u16) -> bool {
    (((a & 0x0FFF) + (b & 0x0FFF)) & 0x1000) != 0
}

fn to_u16(h: u8, l: u8) -> u16 {
    (h as u16) << 8 | l as u16
}
