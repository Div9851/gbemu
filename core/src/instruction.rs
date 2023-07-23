#[derive(Default, Debug, Clone, Copy)]
pub enum Operand8 {
    #[default]
    RegA,
    RegB,
    RegC,
    RegD,
    RegE,
    RegH,
    RegL,
    Imm(u8),
    Address(Operand16),
    IOPortImm(u8), // (0xff00 + n)
    IOPortC,       // (0xff00 + C)
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Operand16 {
    #[default]
    RegBC,
    RegDE,
    RegHL,
    RegSP,
    RegAF,
    Imm(u16),
    AddressImm(u16), // (nn)
}

#[derive(Default, Debug, Clone, Copy)]
pub enum JumpCond {
    #[default]
    NZ,
    Z,
    NC,
    C,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum InstKind {
    #[default]
    Nop,
    Load8(Operand8, Operand8), // A <- B
    LoadIncFromA,
    LoadIncToA,
    LoadDecFromA,
    LoadDecToA,
    Load16(Operand16, Operand16),
    AddAndLoadHL(i8),
    Push(Operand16),
    Pop(Operand16),
    Add8(Operand8),
    AddCarry8(Operand8),
    AddHL(Operand16),
    AddSP(i8),
    Sub8(Operand8),
    SubCarry8(Operand8),
    And8(Operand8),
    Or8(Operand8),
    Xor8(Operand8),
    Compare8(Operand8),
    Inc8(Operand8),
    Dec8(Operand8),
    Inc16(Operand16),
    Dec16(Operand16),
    DecimalAdjustA,
    ComplementA,
    RotateALeft,
    RotateALeftCarry,
    RotateLeft(Operand8),
    RotateLeftCarry(Operand8),
    RotateARight,
    RotateARightCarry,
    RotateRight(Operand8),
    RotateRightCarry(Operand8),
    ShiftLeftArithmetic(Operand8),
    ShiftRightArithmetic(Operand8),
    ShiftRightLogical(Operand8),
    Swap(Operand8),
    TestBit(usize, Operand8),
    SetBit(usize, Operand8),
    ResetBit(usize, Operand8),
    ComplementCarryFlag,
    SetCarryFlag,
    Halt,
    Stop,
    DisableInterrupt,
    EnableInterrupt,
    JumpImm(u16),
    JumpHL,
    JumpCondImm(JumpCond, u16),
    JumpRel(i8),
    JumpCondRel(JumpCond, i8),
    CallImm(u16),
    CallCondImm(JumpCond, u16),
    Return,
    ReturnCond(JumpCond),
    ReturnEnableInterrupt,
    Restart(u16),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Inst {
    pub opcode: usize,
    pub kind: InstKind,
    pub clocks: usize,
    pub length: usize,
}

const OPERANDS8: [Operand8; 8] = [
    Operand8::RegB,
    Operand8::RegC,
    Operand8::RegD,
    Operand8::RegE,
    Operand8::RegH,
    Operand8::RegL,
    Operand8::Address(Operand16::RegHL),
    Operand8::RegA,
];
const OPERANDS16: [Operand16; 4] = [
    Operand16::RegBC,
    Operand16::RegDE,
    Operand16::RegHL,
    Operand16::RegSP,
];

pub fn generate_main_inst_table() -> Vec<Option<Inst>> {
    let conditions = [JumpCond::NZ, JumpCond::Z, JumpCond::NC, JumpCond::C];
    let mut inst_table = vec![None; 256];
    // 8-bit load instructions
    {
        // `LD r,r`, `LD (HL),r` or `LD r, (HL)`
        for (i, op1) in OPERANDS8.iter().enumerate() {
            for (j, op2) in OPERANDS8.iter().enumerate() {
                let opcode = 0x40 + (i << 3) + j;
                if i == 6 && j == 6 {
                    // `LD (HL), (HL)` does not exist. This opcode corresponds to `HALT`.
                    inst_table[opcode] = Some(Inst {
                        opcode,
                        kind: InstKind::Halt,
                        clocks: 4,
                        length: 1,
                    });
                    continue;
                }
                let clocks = if i == 6 || j == 6 { 8 } else { 4 };
                inst_table[opcode] = Some(Inst {
                    opcode,
                    kind: InstKind::Load8(*op1, *op2),
                    clocks,
                    length: 1,
                });
            }
        }
        // `LD r, n` or `LD (HL), n`
        for (i, op1) in OPERANDS8.iter().enumerate() {
            let opcode = 0x06 + (i << 3);
            let clocks = if i == 6 { 12 } else { 8 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Load8(*op1, Operand8::Imm(0)),
                clocks,
                length: 2,
            });
        }
        // `LD A, (BC)`
        inst_table[0x0a] = Some(Inst {
            opcode: 0x0a,
            kind: InstKind::Load8(Operand8::RegA, Operand8::Address(Operand16::RegBC)),
            clocks: 8,
            length: 1,
        });
        // `LD A, (DE)`
        inst_table[0x1a] = Some(Inst {
            opcode: 0x1a,
            kind: InstKind::Load8(Operand8::RegA, Operand8::Address(Operand16::RegDE)),
            clocks: 8,
            length: 1,
        });
        // `LD A, (DE)`
        inst_table[0x1a] = Some(Inst {
            opcode: 0x1a,
            kind: InstKind::Load8(Operand8::RegA, Operand8::Address(Operand16::RegDE)),
            clocks: 8,
            length: 1,
        });
        // `LD A, (nn)`
        inst_table[0xfa] = Some(Inst {
            opcode: 0xfa,
            kind: InstKind::Load8(Operand8::RegA, Operand8::Address(Operand16::Imm(0))),
            clocks: 16,
            length: 3,
        });
        // `LD (BC), A`
        inst_table[0x02] = Some(Inst {
            opcode: 0x02,
            kind: InstKind::Load8(Operand8::Address(Operand16::RegBC), Operand8::RegA),
            clocks: 8,
            length: 1,
        });
        // `LD (DE), A`
        inst_table[0x12] = Some(Inst {
            opcode: 0x12,
            kind: InstKind::Load8(Operand8::Address(Operand16::RegDE), Operand8::RegA),
            clocks: 8,
            length: 1,
        });
        // `LD (nn), A`
        inst_table[0xea] = Some(Inst {
            opcode: 0xea,
            kind: InstKind::Load8(Operand8::Address(Operand16::Imm(0)), Operand8::RegA),
            clocks: 16,
            length: 3,
        });
        // `LD A, (0xff00 + n)`
        inst_table[0xf0] = Some(Inst {
            opcode: 0xf0,
            kind: InstKind::Load8(Operand8::RegA, Operand8::IOPortImm(0)),
            clocks: 12,
            length: 2,
        });
        // `LD (0xff00 + n), A`
        inst_table[0xe0] = Some(Inst {
            opcode: 0xe0,
            kind: InstKind::Load8(Operand8::IOPortImm(0), Operand8::RegA),
            clocks: 12,
            length: 2,
        });
        // `LD A, (0xff00 + C)`
        inst_table[0xf2] = Some(Inst {
            opcode: 0xf2,
            kind: InstKind::Load8(Operand8::RegA, Operand8::IOPortC),
            clocks: 8,
            length: 1,
        });
        // `LD (0xff00 + C), A`
        inst_table[0xe2] = Some(Inst {
            opcode: 0xe2,
            kind: InstKind::Load8(Operand8::IOPortC, Operand8::RegA),
            clocks: 8,
            length: 1,
        });
        // `LDI (HL), A`
        inst_table[0x22] = Some(Inst {
            opcode: 0x22,
            kind: InstKind::LoadIncFromA,
            clocks: 8,
            length: 1,
        });
        // `LDI A, (HL)`
        inst_table[0x2a] = Some(Inst {
            opcode: 0x2a,
            kind: InstKind::LoadIncToA,
            clocks: 8,
            length: 1,
        });
        // `LDD (HL), A`
        inst_table[0x32] = Some(Inst {
            opcode: 0x32,
            kind: InstKind::LoadDecFromA,
            clocks: 8,
            length: 1,
        });
        // `LDD A, (HL)`
        inst_table[0x3a] = Some(Inst {
            opcode: 0x3a,
            kind: InstKind::LoadDecToA,
            clocks: 8,
            length: 1,
        });
    }
    // 16-bit load instructions
    {
        // `LD, rr, nn`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 1 + (i << 4);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Load16(*op, Operand16::Imm(0)),
                clocks: 12,
                length: 3,
            })
        }
        // `LD (nn), SP`
        inst_table[0x08] = Some(Inst {
            opcode: 0x08,
            kind: InstKind::Load16(Operand16::AddressImm(0), Operand16::RegSP),
            clocks: 20,
            length: 3,
        });
        // `LD SP, HL`
        inst_table[0xf9] = Some(Inst {
            opcode: 0xf9,
            kind: InstKind::Load16(Operand16::RegSP, Operand16::RegHL),
            clocks: 8,
            length: 1,
        });
        // `PUSH rr, nn`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 0xc5 + (i << 4);
            if i == 3 {
                // `PUSH SP, nn` does not exist. This opcode corresponds to `PUSH AF, nn`.
                inst_table[opcode] = Some(Inst {
                    opcode,
                    kind: InstKind::Push(Operand16::RegAF),
                    clocks: 16,
                    length: 1,
                });
                continue;
            }
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Push(*op),
                clocks: 16,
                length: 1,
            })
        }
        // `POP rr, nn`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 0xc1 + (i << 4);
            if i == 3 {
                // `PUSH SP, nn` does not exist. This opcode corresponds to `PUSH AF, nn`.
                inst_table[opcode] = Some(Inst {
                    opcode,
                    kind: InstKind::Pop(Operand16::RegAF),
                    clocks: 12,
                    length: 1,
                });
                continue;
            }
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Pop(*op),
                clocks: 12,
                length: 1,
            })
        }
    }
    // 8-bit arithmetic/logic instructions
    {
        // `ADD A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x80 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Add8(*op),
                clocks,
                length: 1,
            });
        }
        // `ADD A, d`
        inst_table[0xc6] = Some(Inst {
            opcode: 0xc6,
            kind: InstKind::Add8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `ADC A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x88 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::AddCarry8(*op),
                clocks,
                length: 1,
            });
        }
        // `ADC A, d`
        inst_table[0xce] = Some(Inst {
            opcode: 0xce,
            kind: InstKind::AddCarry8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `SUB A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x90 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Sub8(*op),
                clocks,
                length: 1,
            });
        }
        // `SUB A, d`
        inst_table[0xd6] = Some(Inst {
            opcode: 0xd6,
            kind: InstKind::Sub8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `SBC A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x98 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::SubCarry8(*op),
                clocks,
                length: 1,
            });
        }
        // `SBC A, d`
        inst_table[0xde] = Some(Inst {
            opcode: 0xde,
            kind: InstKind::SubCarry8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `AND A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0xa0 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::And8(*op),
                clocks,
                length: 1,
            });
        }
        // `AND A, d`
        inst_table[0xe6] = Some(Inst {
            opcode: 0xe6,
            kind: InstKind::And8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `XOR A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0xa8 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Xor8(*op),
                clocks,
                length: 1,
            });
        }
        // `XOR A, d`
        inst_table[0xee] = Some(Inst {
            opcode: 0xee,
            kind: InstKind::Xor8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `OR A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0xb0 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Or8(*op),
                clocks,
                length: 1,
            });
        }
        // `OR A, d`
        inst_table[0xf6] = Some(Inst {
            opcode: 0xf6,
            kind: InstKind::Or8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `CP A, r`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0xb8 + i;
            let clocks = if i == 6 { 8 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Compare8(*op),
                clocks,
                length: 1,
            });
        }
        // `CP A, d`
        inst_table[0xfe] = Some(Inst {
            opcode: 0xfe,
            kind: InstKind::Compare8(Operand8::Imm(0)),
            clocks: 8,
            length: 2,
        });
        // `INC r` or `INC (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x04 + (i << 3);
            let clocks = if i == 6 { 12 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Inc8(*op),
                clocks,
                length: 1,
            });
        }
        // `DEC r` or `DEC (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x05 + (i << 3);
            let clocks = if i == 6 { 12 } else { 4 };
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Dec8(*op),
                clocks,
                length: 1,
            });
        }
        // `DAA`
        inst_table[0x27] = Some(Inst {
            opcode: 0x27,
            kind: InstKind::DecimalAdjustA,
            clocks: 4,
            length: 1,
        });
        // `CPL`
        inst_table[0x2f] = Some(Inst {
            opcode: 0x2f,
            kind: InstKind::ComplementA,
            clocks: 4,
            length: 1,
        });
    }
    // 16-bit arithmetic/logic instructions
    {
        // `ADD HL, rr`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 0x09 + (i << 4);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::AddHL(*op),
                clocks: 8,
                length: 1,
            });
        }
        // `INC rr`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 0x03 + (i << 4);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Inc16(*op),
                clocks: 8,
                length: 1,
            });
        }
        // `DEC rr`
        for (i, op) in OPERANDS16.iter().enumerate() {
            let opcode = 0x0b + (i << 4);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Dec16(*op),
                clocks: 8,
                length: 1,
            });
        }
        // `ADD SP, dd`
        inst_table[0xe8] = Some(Inst {
            opcode: 0xe8,
            kind: InstKind::AddSP(0),
            clocks: 16,
            length: 2,
        });
        // `LD HL, SP + dd`
        inst_table[0xf8] = Some(Inst {
            opcode: 0xf8,
            kind: InstKind::AddAndLoadHL(0),
            clocks: 12,
            length: 2,
        });
    }
    // rotate and shift instructions
    {
        // `RLCA`
        inst_table[0x07] = Some(Inst {
            opcode: 0x07,
            kind: InstKind::RotateALeft,
            clocks: 4,
            length: 1,
        });
        // `RLA`
        inst_table[0x17] = Some(Inst {
            opcode: 0x17,
            kind: InstKind::RotateALeftCarry,
            clocks: 4,
            length: 1,
        });
        // `RRCA`
        inst_table[0x0f] = Some(Inst {
            opcode: 0x0f,
            kind: InstKind::RotateARight,
            clocks: 4,
            length: 1,
        });
        // RRA
        inst_table[0x1f] = Some(Inst {
            opcode: 0x1f,
            kind: InstKind::RotateARightCarry,
            clocks: 4,
            length: 1,
        });
    }
    // CPU control instructions
    {
        // `CCF`
        inst_table[0x3f] = Some(Inst {
            opcode: 0x3f,
            kind: InstKind::ComplementCarryFlag,
            clocks: 4,
            length: 1,
        });
        // `SCF`
        inst_table[0x37] = Some(Inst {
            opcode: 0x37,
            kind: InstKind::SetCarryFlag,
            clocks: 4,
            length: 1,
        });
        // `NOP`
        inst_table[0x00] = Some(Inst {
            opcode: 0x00,
            kind: InstKind::Nop,
            clocks: 4,
            length: 1,
        });
        // `NOP`
        inst_table[0x00] = Some(Inst {
            opcode: 0x00,
            kind: InstKind::Nop,
            clocks: 4,
            length: 1,
        });
        // `STOP`
        inst_table[0x10] = Some(Inst {
            opcode: 0x10,
            kind: InstKind::Stop,
            clocks: 4,
            length: 2,
        });
        // `DI`
        inst_table[0xf3] = Some(Inst {
            opcode: 0xf3,
            kind: InstKind::DisableInterrupt,
            clocks: 4,
            length: 1,
        });
        // `EI`
        inst_table[0xfb] = Some(Inst {
            opcode: 0xfb,
            kind: InstKind::EnableInterrupt,
            clocks: 4,
            length: 1,
        });
    }
    // jump instructions
    {
        // `JP nn`
        inst_table[0xc3] = Some(Inst {
            opcode: 0xc3,
            kind: InstKind::JumpImm(0),
            clocks: 16,
            length: 3,
        });
        // `JP HL`
        inst_table[0xe9] = Some(Inst {
            opcode: 0xe9,
            kind: InstKind::JumpHL,
            clocks: 4,
            length: 1,
        });
        // `JP f, nn`
        for (i, cond) in conditions.iter().enumerate() {
            let opcode = 0xc2 + (i << 3);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::JumpCondImm(*cond, 0),
                clocks: 12,
                length: 3,
            });
        }
        // `JR PC+dd`
        inst_table[0x18] = Some(Inst {
            opcode: 0x18,
            kind: InstKind::JumpRel(0),
            clocks: 12,
            length: 2,
        });
        // `JR f, PC+dd`
        for (i, cond) in conditions.iter().enumerate() {
            let opcode = 0x20 + (i << 3);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::JumpCondRel(*cond, 0),
                clocks: 8,
                length: 2,
            });
        }
        // `CALL nn`
        inst_table[0xcd] = Some(Inst {
            opcode: 0xcd,
            kind: InstKind::CallImm(0),
            clocks: 24,
            length: 3,
        });
        // `CALL f, nn`
        for (i, cond) in conditions.iter().enumerate() {
            let opcode = 0xc4 + (i << 3);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::CallCondImm(*cond, 0),
                clocks: 12,
                length: 3,
            });
        }
        // `RET`
        inst_table[0xc9] = Some(Inst {
            opcode: 0xc9,
            kind: InstKind::Return,
            clocks: 16,
            length: 1,
        });
        // `RET f`
        for (i, cond) in conditions.iter().enumerate() {
            let opcode = 0xc0 + (i << 3);
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::ReturnCond(*cond),
                clocks: 8,
                length: 1,
            });
        }
        // `RETI`
        inst_table[0xd9] = Some(Inst {
            opcode: 0xd9,
            kind: InstKind::ReturnEnableInterrupt,
            clocks: 16,
            length: 1,
        });
        // `RST n`
        let mut return_address = 0x00;
        while return_address <= 0x38 {
            let opcode = 0xc7 + return_address;
            inst_table[opcode] = Some(Inst {
                opcode,
                kind: InstKind::Restart(return_address as u16),
                clocks: 16,
                length: 1,
            });
            return_address += 0x08;
        }
    }
    inst_table
}

pub fn generate_sub_inst_table() -> Vec<Inst> {
    let mut inst_table = vec![Default::default(); 256];
    // rotate and shift instructions
    {
        // `RLC r` or `RLC (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::RotateLeft(*op),
                clocks,
                length: 2,
            };
        }
        // `RRC r` or `RRC (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x08 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::RotateRight(*op),
                clocks,
                length: 2,
            };
        }
        // `RL r` or `RL (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x10 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::RotateLeftCarry(*op),
                clocks,
                length: 2,
            };
        }
        // `RR r` or `RR (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x18 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::RotateRightCarry(*op),
                clocks,
                length: 2,
            };
        }
        // `SLA r` or `SLA (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x20 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::ShiftLeftArithmetic(*op),
                clocks,
                length: 2,
            };
        }
        // `SRA r` or `SRA (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x28 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::ShiftRightArithmetic(*op),
                clocks,
                length: 2,
            };
        }
        // `SWAP r` or `SWAP r (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x30 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::Swap(*op),
                clocks,
                length: 2,
            };
        }
        // `SRL r` or `SRL (HL)`
        for (i, op) in OPERANDS8.iter().enumerate() {
            let opcode = 0x38 + i;
            let clocks = if i == 6 { 16 } else { 8 };
            inst_table[opcode] = Inst {
                opcode,
                kind: InstKind::ShiftRightLogical(*op),
                clocks,
                length: 2,
            };
        }
    }
    // single-bit operation instructions
    {
        // `BIT n, r` or `BIT n, (HL)`
        for n in 0..8 {
            for (i, op) in OPERANDS8.iter().enumerate() {
                let opcode = 0x40 + (n << 3) + i;
                let clocks = if i == 6 { 12 } else { 8 };
                inst_table[opcode] = Inst {
                    opcode,
                    kind: InstKind::TestBit(n, *op),
                    clocks,
                    length: 2,
                };
            }
        }
        // `RES n, r` or `RES n, (HL)`
        for n in 0..8 {
            for (i, op) in OPERANDS8.iter().enumerate() {
                let opcode = 0x80 + (n << 3) + i;
                let clocks = if i == 6 { 16 } else { 8 };
                inst_table[opcode] = Inst {
                    opcode,
                    kind: InstKind::ResetBit(n, *op),
                    clocks,
                    length: 2,
                };
            }
        }
        // `SET n, r` or `SET n, (HL)`
        for n in 0..8 {
            for (i, op) in OPERANDS8.iter().enumerate() {
                let opcode = 0xc0 + (n << 3) + i;
                let clocks = if i == 6 { 16 } else { 8 };
                inst_table[opcode] = Inst {
                    opcode,
                    kind: InstKind::SetBit(n, *op),
                    clocks,
                    length: 2,
                };
            }
        }
    }
    inst_table
}
