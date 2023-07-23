use crate::instruction::{self, Inst, InstKind, JumpCond, Operand16, Operand8};
use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const INTERRUPT_HANDLER: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

pub struct CPU {
    pub registers: Registers,
    pub memory: Rc<RefCell<Memory>>,
    pub prev_inst: Option<InstKind>,
    pub current_inst: Option<InstKind>,
    pub clocks_to_finish: usize,
    pub main_inst_table: Vec<Option<Inst>>,
    pub sub_inst_table: Vec<Inst>,
    pub is_halt: bool,
    pub is_halt_bug_occured: bool,
}

impl CPU {
    pub fn new(memory: Rc<RefCell<Memory>>) -> CPU {
        let main_inst_table = instruction::generate_main_inst_table();
        let sub_inst_table = instruction::generate_sub_inst_table();
        CPU {
            registers: Default::default(),
            memory,
            prev_inst: None,
            current_inst: None,
            clocks_to_finish: 0,
            main_inst_table,
            sub_inst_table,
            is_halt: false,
            is_halt_bug_occured: false,
        }
    }

    pub fn decode(&mut self) -> Inst {
        let opcode = self.memory.borrow().get_byte(self.registers.pc);
        if self.is_halt_bug_occured {
            self.is_halt_bug_occured = false;
        } else {
            self.registers.pc += 1;
        }
        let inst = if opcode == 0xcb {
            let opcode = self.memory.borrow().get_byte(self.registers.pc);
            self.registers.pc += 1;
            self.sub_inst_table[opcode as usize]
        } else {
            match self.main_inst_table[opcode as usize] {
                Some(inst) => inst,
                None => panic!("unknown opcode {}", opcode),
            }
        };
        match inst.kind {
            InstKind::Load8(dst, Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Load8(dst, Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::Load8(dst, Operand8::Address(Operand16::Imm(_))) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::Load8(dst, Operand8::Address(Operand16::Imm(n))),
                    ..inst
                }
            }
            InstKind::Load8(Operand8::Address(Operand16::Imm(_)), src) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::Load8(Operand8::Address(Operand16::Imm(n)), src),
                    ..inst
                }
            }
            InstKind::Load8(dst, Operand8::IOPortImm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Load8(dst, Operand8::IOPortImm(n)),
                    ..inst
                }
            }
            InstKind::Load8(Operand8::IOPortImm(_), src) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Load8(Operand8::IOPortImm(n), src),
                    ..inst
                }
            }
            InstKind::Load16(dst, Operand16::Imm(_)) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::Load16(dst, Operand16::Imm(n)),
                    ..inst
                }
            }
            InstKind::Load16(Operand16::AddressImm(_), src) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::Load16(Operand16::AddressImm(n), src),
                    ..inst
                }
            }
            InstKind::Add8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Add8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::AddCarry8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::AddCarry8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::Sub8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Sub8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::SubCarry8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::SubCarry8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::And8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::And8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::Xor8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Xor8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::Or8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Or8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::Compare8(Operand8::Imm(_)) => {
                let n = self.memory.borrow().get_byte(self.registers.pc);
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::Compare8(Operand8::Imm(n)),
                    ..inst
                }
            }
            InstKind::AddSP(_) => {
                let n = self.memory.borrow().get_byte(self.registers.pc) as i8;
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::AddSP(n),
                    ..inst
                }
            }
            InstKind::AddAndLoadHL(_) => {
                let n = self.memory.borrow().get_byte(self.registers.pc) as i8;
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::AddAndLoadHL(n),
                    ..inst
                }
            }
            InstKind::JumpImm(_) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::JumpImm(n),
                    ..inst
                }
            }
            InstKind::JumpCondImm(cond, _) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::JumpCondImm(cond, n),
                    ..inst
                }
            }
            InstKind::JumpRel(_) => {
                let n = self.memory.borrow().get_byte(self.registers.pc) as i8;
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::JumpRel(n),
                    ..inst
                }
            }
            InstKind::JumpCondRel(cond, _) => {
                let n = self.memory.borrow().get_byte(self.registers.pc) as i8;
                self.registers.pc += 1;
                Inst {
                    kind: InstKind::JumpCondRel(cond, n),
                    ..inst
                }
            }
            InstKind::CallImm(_) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::CallImm(n),
                    ..inst
                }
            }
            InstKind::CallCondImm(cond, _) => {
                let n = self.memory.borrow().get_word(self.registers.pc);
                self.registers.pc += 2;
                Inst {
                    kind: InstKind::CallCondImm(cond, n),
                    ..inst
                }
            }
            _ => inst,
        }
    }

    fn get8(&self, op: Operand8) -> u8 {
        match op {
            Operand8::RegA => self.registers.a,
            Operand8::RegB => self.registers.b,
            Operand8::RegC => self.registers.c,
            Operand8::RegD => self.registers.d,
            Operand8::RegE => self.registers.e,
            Operand8::RegH => self.registers.h,
            Operand8::RegL => self.registers.l,
            Operand8::Imm(n) => n,
            Operand8::Address(op16) => {
                let addr = self.get16(op16);
                self.memory.borrow().get_byte(addr)
            }
            Operand8::IOPortImm(n) => {
                let addr = 0xff00 + n as u16;
                self.memory.borrow().get_byte(addr)
            }
            Operand8::IOPortC => {
                let addr = 0xff00 + self.registers.c as u16;
                self.memory.borrow().get_byte(addr)
            }
        }
    }

    fn get16(&self, op: Operand16) -> u16 {
        match op {
            Operand16::RegBC => self.registers.get_bc(),
            Operand16::RegDE => self.registers.get_de(),
            Operand16::RegHL => self.registers.get_hl(),
            Operand16::RegSP => self.registers.sp,
            Operand16::RegAF => self.registers.get_af(),
            Operand16::Imm(value) => value,
            _ => unreachable!(),
        }
    }

    fn set8(&mut self, op: Operand8, value: u8) {
        match op {
            Operand8::RegA => self.registers.a = value,
            Operand8::RegB => self.registers.b = value,
            Operand8::RegC => self.registers.c = value,
            Operand8::RegD => self.registers.d = value,
            Operand8::RegE => self.registers.e = value,
            Operand8::RegH => self.registers.h = value,
            Operand8::RegL => self.registers.l = value,
            Operand8::Address(op16) => {
                let addr = self.get16(op16);
                self.memory.borrow_mut().set_byte(addr, value);
            }
            Operand8::IOPortImm(n) => {
                let addr = 0xff00 + n as u16;
                self.memory.borrow_mut().set_byte(addr, value);
            }
            Operand8::IOPortC => {
                let addr = 0xff00 + self.registers.c as u16;
                self.memory.borrow_mut().set_byte(addr, value);
            }
            _ => unreachable!(),
        }
    }

    fn set16(&mut self, op: Operand16, value: u16) {
        match op {
            Operand16::RegBC => self.registers.set_bc(value),
            Operand16::RegDE => self.registers.set_de(value),
            Operand16::RegHL => self.registers.set_hl(value),
            Operand16::RegSP => self.registers.sp = value,
            Operand16::RegAF => self.registers.set_af(value),
            Operand16::AddressImm(addr) => {
                self.memory.borrow_mut().set_word(addr, value);
            }
            _ => unreachable!(),
        }
    }

    pub fn execute(&mut self, inst: InstKind) {
        match inst {
            InstKind::Nop => {}
            InstKind::Load8(dst, src) => {
                let value = self.get8(src);
                self.set8(dst, value);
            }
            InstKind::LoadIncFromA => {
                let hl = self.registers.get_hl();
                let value = self.registers.a;
                self.memory.borrow_mut().set_byte(hl, value);
                self.registers.set_hl(hl + 1);
            }
            InstKind::LoadIncToA => {
                let hl = self.registers.get_hl();
                let value = self.memory.borrow().get_byte(hl);
                self.registers.a = value;
                self.registers.set_hl(hl + 1);
            }
            InstKind::LoadDecFromA => {
                let hl = self.registers.get_hl();
                let value = self.registers.a;
                self.memory.borrow_mut().set_byte(hl, value);
                self.registers.set_hl(hl - 1);
            }
            InstKind::LoadDecToA => {
                let hl = self.registers.get_hl();
                let value = self.memory.borrow().get_byte(hl);
                self.registers.a = value;
                self.registers.set_hl(hl - 1);
            }
            InstKind::Load16(dst, src) => {
                let value = self.get16(src);
                self.set16(dst, value);
            }
            InstKind::AddAndLoadHL(n) => {
                // 00hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.sp;
                let b = n as u16; // sign extension
                let value = a.wrapping_add(b);
                let carry = ((a & 0xff) + (b & 0xff)) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf)) > 0xf;
                self.registers.set_hl(value);
                flags.z = false;
                flags.n = false;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::Push(op) => {
                let value = self.get16(op);
                self.registers.sp -= 2;
                self.memory.borrow_mut().set_word(self.registers.sp, value);
            }
            InstKind::Pop(op) => {
                let value = self.memory.borrow_mut().get_word(self.registers.sp);
                self.registers.sp += 2;
                self.set16(op, value);
            }
            InstKind::Add8(op) => {
                // z0hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_add(b);
                let half_carry = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = false;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::AddCarry8(op) => {
                // z0hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a as u16;
                let b = self.get8(op) as u16;
                let c = if flags.c { 1 } else { 0 };
                let value = (a.wrapping_add(b + c) & 0xff) as u8;
                let carry = (a + b + c) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf) + c) > 0xf;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = false;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::AddHL(op) => {
                // -0hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.get_hl();
                let b = self.get16(op);
                let (value, carry) = a.overflowing_add(b);
                let half_carry = ((a & 0xfff) + (b & 0xfff)) > 0xfff;
                self.registers.set_hl(value);
                flags.n = false;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::AddSP(n) => {
                // 00hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.sp;
                let b = n as u16; // sign extension
                let value = a.wrapping_add(b);
                let carry = ((a & 0xff) + (b & 0xff)) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf)) > 0xf;
                self.registers.sp = value;
                flags.z = false;
                flags.n = false;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::Sub8(op) => {
                // z1hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_sub(b);
                let half_carry = (a & 0xf) < (b & 0xf);
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = true;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::SubCarry8(op) => {
                // z1hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a as u16;
                let b = self.get8(op) as u16;
                let c = if flags.c { 1 } else { 0 };
                let value = (a.wrapping_sub(b + c) & 0xff) as u8;
                let carry = a < (b + c);
                let half_carry = (a & 0xf) < (b & 0xf) + c;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = true;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::And8(op) => {
                // z010
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a & b;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = false;
                flags.h = true;
                flags.c = false;
                self.registers.f = flags.into();
            }
            InstKind::Or8(op) => {
                // z000
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a | b;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = false;
                self.registers.f = flags.into();
            }
            InstKind::Xor8(op) => {
                // z000
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a ^ b;
                self.registers.a = value;
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = false;
                self.registers.f = flags.into();
            }
            InstKind::Compare8(op) => {
                // z1hc
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_sub(b);
                let half_carry = (a & 0xf) < (b & 0xf);
                flags.z = value == 0;
                flags.n = true;
                flags.h = half_carry;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::Inc8(op) => {
                // z0h-
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let (value, _) = a.overflowing_add(1);
                let half_carry = ((a & 0xf) + 1) & 0x10 == 0x10;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = half_carry;
                self.registers.f = flags.into();
            }
            InstKind::Dec8(op) => {
                // z1h-
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let (value, _) = a.overflowing_sub(1);
                let half_carry = (a & 0xf) < 1;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = true;
                flags.h = half_carry;
                self.registers.f = flags.into();
            }
            InstKind::Inc16(op) => {
                let a = self.get16(op);
                let value = a.wrapping_add(1);
                self.set16(op, value);
            }
            InstKind::Dec16(op) => {
                let a = self.get16(op);
                let value = a.wrapping_sub(1);
                self.set16(op, value);
            }
            InstKind::DecimalAdjustA => {
                // z-0c
                //ref: https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let mut correction = 0;
                let mut carry = false;
                if flags.h || (!flags.n && (a & 0xf) > 0x9) {
                    correction += 0x6;
                }
                if flags.c || (!flags.n && a > 0x99) {
                    correction += 0x60;
                    carry = true;
                }
                let value = if flags.n {
                    self.registers.a.wrapping_sub(correction)
                } else {
                    self.registers.a.wrapping_add(correction)
                };
                self.registers.a = value;
                flags.z = value == 0;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::ComplementA => {
                // -11-
                let mut flags = Flags::from(self.registers.f);
                self.registers.a ^= 0xff;
                flags.n = true;
                flags.h = true;
                self.registers.f = flags.into();
            }
            InstKind::RotateALeft => {
                // 000c
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let value = ((a & 0x7f) << 1) + ((a >> 7) & 1);
                let carry = (a & (1 << 7)) != 0;
                self.registers.a = value;
                flags.z = false;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateALeftCarry => {
                // 000c
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let c = if flags.c { 1 } else { 0 };
                let value = ((a & 0x7f) << 1) + c;
                let carry = (a & (1 << 7)) != 0;
                self.registers.a = value;
                flags.z = false;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateARight => {
                // 000c
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let value = ((a & 0xfe) >> 1) + ((a & 1) << 7);
                let carry = (a & 1) != 0;
                self.registers.a = value;
                flags.z = false;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateARightCarry => {
                // 000c
                let mut flags = Flags::from(self.registers.f);
                let a = self.registers.a;
                let c = if flags.c { 1 } else { 0 };
                let value = ((a & 0xfe) >> 1) + (c << 7);
                let carry = (a & 1) != 0;
                self.registers.a = value;
                flags.z = false;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateLeft(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let value = ((a & 0x7f) << 1) + ((a >> 7) & 1);
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateLeftCarry(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let c = if flags.c { 1 } else { 0 };
                let value = ((a & 0x7f) << 1) + c;
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateRight(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let value = ((a & 0xfe) >> 1) + ((a & 1) << 7);
                let carry = (a & 1) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::RotateRightCarry(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let c = if flags.c { 1 } else { 0 };
                let value = ((a & 0xfe) >> 1) + (c << 7);
                let carry = (a & 1) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::ShiftLeftArithmetic(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let value = (a & 0x7f) << 1;
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::ShiftRightArithmetic(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let msb = a & (1 << 7);
                let value = ((a & 0xfe) >> 1) + msb;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = (a & 1) != 0;
                self.registers.f = flags.into();
            }
            InstKind::ShiftRightLogical(op) => {
                // z00c
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let value = (a & 0xfe) >> 1;
                let carry = (a & 1) != 0;
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = carry;
                self.registers.f = flags.into();
            }
            InstKind::Swap(op) => {
                // z000
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                let value = ((a & 0xf0) >> 4) + ((a & 0xf) << 4);
                self.set8(op, value);
                flags.z = value == 0;
                flags.n = false;
                flags.h = false;
                flags.c = false;
                self.registers.f = flags.into();
            }
            InstKind::TestBit(n, op) => {
                // z01-
                let mut flags = Flags::from(self.registers.f);
                let a = self.get8(op);
                flags.z = ((a >> n) & 1) == 0;
                flags.n = false;
                flags.h = true;
                self.registers.f = flags.into();
            }
            InstKind::SetBit(n, op) => {
                let a = self.get8(op);
                let value = a | (1 << n);
                self.set8(op, value);
            }
            InstKind::ResetBit(n, op) => {
                let a = self.get8(op);
                let value = a & (0xff ^ (1 << n));
                self.set8(op, value);
            }
            InstKind::ComplementCarryFlag => {
                // -00c
                let mut flags = Flags::from(self.registers.f);
                flags.n = false;
                flags.h = false;
                flags.c ^= true;
                self.registers.f = flags.into();
            }
            InstKind::SetCarryFlag => {
                // -001
                let mut flags = Flags::from(self.registers.f);
                flags.n = false;
                flags.h = false;
                flags.c = true;
                self.registers.f = flags.into();
            }
            InstKind::Halt => {
                if self.is_halt {
                    return;
                }
                let interrupt = self.memory.borrow().interrupt_flag
                    & self.memory.borrow().interrupt_enable
                    & 0x1f;
                if !self.memory.borrow().interrupt_master_enable && interrupt != 0 {
                    // HALT bug occurs!
                    self.is_halt_bug_occured = true;
                } else {
                    self.is_halt = true;
                }
            }
            InstKind::Stop => {}
            InstKind::DisableInterrupt => {
                self.memory.borrow_mut().interrupt_master_enable = false;
            }
            InstKind::EnableInterrupt => {} // the effect of EI is delayed by one instruction
            InstKind::JumpImm(addr) => {
                self.registers.pc = addr;
            }
            InstKind::JumpHL => {
                let addr = self.registers.get_hl();
                self.registers.pc = addr;
            }
            InstKind::JumpCondImm(cond, addr) => {
                let flags = Flags::from(self.registers.f);
                let jump_cond = match cond {
                    JumpCond::NZ => !flags.z,
                    JumpCond::Z => flags.z,
                    JumpCond::NC => !flags.c,
                    JumpCond::C => flags.c,
                };
                if jump_cond {
                    // if jump condition is met, convert it to unconditional jump
                    self.current_inst = Some(InstKind::JumpImm(addr));
                    self.clocks_to_finish = 4;
                }
            }
            InstKind::JumpRel(offset) => {
                let addr = self.registers.pc.wrapping_add(offset as u16);
                self.registers.pc = addr;
            }
            InstKind::JumpCondRel(cond, offset) => {
                let flags = Flags::from(self.registers.f);
                let jump_cond = match cond {
                    JumpCond::NZ => !flags.z,
                    JumpCond::Z => flags.z,
                    JumpCond::NC => !flags.c,
                    JumpCond::C => flags.c,
                };
                if jump_cond {
                    // if jump condition is met, convert it to unconditional jump
                    self.current_inst = Some(InstKind::JumpRel(offset));
                    self.clocks_to_finish = 4;
                }
            }
            InstKind::CallImm(addr) => {
                self.registers.sp -= 2;
                self.memory
                    .borrow_mut()
                    .set_word(self.registers.sp, self.registers.pc);
                self.registers.pc = addr;
            }
            InstKind::CallCondImm(cond, addr) => {
                let flags = Flags::from(self.registers.f);
                let jump_cond = match cond {
                    JumpCond::NZ => !flags.z,
                    JumpCond::Z => flags.z,
                    JumpCond::NC => !flags.c,
                    JumpCond::C => flags.c,
                };
                if jump_cond {
                    // if jump condition is met, convert it to unconditional call
                    self.current_inst = Some(InstKind::CallImm(addr));
                    self.clocks_to_finish = 12;
                }
            }
            InstKind::Return => {
                let addr = self.memory.borrow().get_word(self.registers.sp);
                self.registers.pc = addr;
                self.registers.sp += 2;
            }
            InstKind::ReturnCond(cond) => {
                let flags = Flags::from(self.registers.f);
                let jump_cond = match cond {
                    JumpCond::NZ => !flags.z,
                    JumpCond::Z => flags.z,
                    JumpCond::NC => !flags.c,
                    JumpCond::C => flags.c,
                };
                if jump_cond {
                    // if jump condition is met, convert it to unconditional return
                    self.current_inst = Some(InstKind::Return);
                    self.clocks_to_finish = 12;
                }
            }
            InstKind::ReturnEnableInterrupt => {
                let addr = self.memory.borrow().get_word(self.registers.sp);
                self.registers.pc = addr;
                self.registers.sp += 2;
                self.memory.borrow_mut().interrupt_master_enable = true;
            }
            InstKind::Restart(addr) => {
                self.registers.sp -= 2;
                self.memory
                    .borrow_mut()
                    .set_word(self.registers.sp, self.registers.pc);
                self.registers.pc = addr;
            }
        }
    }

    pub fn tick(&mut self) {
        if self.current_inst.is_none() {
            let interrupt =
                self.memory.borrow().interrupt_flag & self.memory.borrow().interrupt_enable & 0x1f;
            if self.is_halt && interrupt == 0 {
                self.current_inst = Some(InstKind::Halt);
                self.clocks_to_finish = 4;
            } else if self.memory.borrow().interrupt_master_enable && interrupt != 0 {
                for (i, addr) in INTERRUPT_HANDLER.iter().enumerate() {
                    if interrupt & (1 << i) != 0 {
                        self.memory.borrow_mut().interrupt_master_enable = false;
                        self.memory.borrow_mut().interrupt_flag ^= 1 << i;
                        self.current_inst = Some(InstKind::CallImm(*addr));
                        self.clocks_to_finish = 20;
                        break;
                    }
                }
                self.is_halt = false;
            } else {
                let inst = self.decode();
                self.current_inst = Some(inst.kind);
                self.clocks_to_finish = inst.clocks;
                self.is_halt = false;
            }
        }
        self.clocks_to_finish -= 1;
        if self.clocks_to_finish == 0 {
            let inst = self.current_inst.take().unwrap();
            self.execute(inst);
            if let Some(InstKind::EnableInterrupt) = self.prev_inst {
                self.memory.borrow_mut().interrupt_master_enable = true;
            }
            self.prev_inst = Some(inst);
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8, // only upper 4 bits are used and lower 4 bits should be zero.
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        return ((self.a as u16) << 8) | self.f as u16;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xf0) as u8;
    }

    pub fn get_bc(&self) -> u16 {
        return ((self.b as u16) << 8) | self.c as u16;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    pub fn get_de(&self) -> u16 {
        return ((self.d as u16) << 8) | self.e as u16;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        return ((self.h as u16) << 8) | self.l as u16;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Flags {
            z: (value & (1 << 7)) != 0,
            n: (value & (1 << 6)) != 0,
            h: (value & (1 << 5)) != 0,
            c: (value & (1 << 4)) != 0,
        }
    }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 {
        let mut value = 0;
        if self.z {
            value |= 1 << 7;
        }
        if self.n {
            value |= 1 << 6;
        }
        if self.h {
            value |= 1 << 5;
        }
        if self.c {
            value |= 1 << 4;
        }
        value
    }
}
