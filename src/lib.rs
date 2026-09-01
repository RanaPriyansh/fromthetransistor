//! Tiny 8-register CPU emulator.
//!
//! Educational slice only: fetch/decode/execute for a seven-op toy ISA.
//! Not a tape-out and not the geohot/fromthetransistor curriculum.

use std::fmt;

/// Number of general-purpose registers.
pub const GPR_COUNT: usize = 8;

/// 16-bit instruction word.
pub type Word = u16;

/// Opcode in bits 15–12 of a [`Word`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    /// Stop the CPU. Register fields are ignored.
    Halt = 0,
    /// `rd = rs1 wrapping_add rs2`
    Add = 1,
    /// `rd = rs1 wrapping_sub rs2`
    Sub = 2,
    /// `rd = rs1 & rs2`
    And = 3,
    /// `rd = rs1 | rs2`
    Or = 4,
    /// `rd = rs1 ^ rs2`
    Xor = 5,
    /// `rd = rs1` (`rs2` ignored)
    Mov = 6,
}

impl Opcode {
    /// Parse the 4-bit opcode field. Unknown values are `Err`.
    pub fn from_u8(bits: u8) -> Result<Self, Error> {
        match bits {
            0 => Ok(Self::Halt),
            1 => Ok(Self::Add),
            2 => Ok(Self::Sub),
            3 => Ok(Self::And),
            4 => Ok(Self::Or),
            5 => Ok(Self::Xor),
            6 => Ok(Self::Mov),
            other => Err(Error::UnknownOpcode(other)),
        }
    }
}

/// Decoded instruction. Register indexes are always `0..8`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Operation.
    pub op: Opcode,
    /// Destination GPR.
    pub rd: u8,
    /// First source GPR.
    pub rs1: u8,
    /// Second source GPR (ignored by `HALT` and `MOV`).
    pub rs2: u8,
}

/// Decode / execute failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Opcode nibble is not one of the seven ops.
    UnknownOpcode(u8),
    /// `pc` is past the end of instruction memory.
    PcOutOfRange {
        /// Program counter (instruction index).
        pc: usize,
        /// Length of instruction memory.
        len: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownOpcode(op) => write!(f, "unknown opcode {op:#x}"),
            Error::PcOutOfRange { pc, len } => {
                write!(f, "pc {pc} out of range (len {len})")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Pack an instruction into the documented 16-bit layout.
pub fn encode(inst: Instruction) -> Word {
    ((inst.op as u16) << 12)
        | ((u16::from(inst.rd) & 0b111) << 9)
        | ((u16::from(inst.rs1) & 0b111) << 6)
        | ((u16::from(inst.rs2) & 0b111) << 3)
}

/// Unpack a 16-bit word. Bits 2–0 are ignored.
pub fn decode(word: Word) -> Result<Instruction, Error> {
    let op = Opcode::from_u8(((word >> 12) & 0xF) as u8)?;
    let rd = ((word >> 9) & 0b111) as u8;
    let rs1 = ((word >> 6) & 0b111) as u8;
    let rs2 = ((word >> 3) & 0b111) as u8;
    Ok(Instruction { op, rd, rs1, rs2 })
}

/// Eight GPRs, a program counter, and instruction memory.
#[derive(Clone, Debug)]
pub struct Cpu {
    regs: [u8; GPR_COUNT],
    pc: usize,
    halted: bool,
    mem: Vec<Word>,
}

impl Cpu {
    /// New CPU with zeroed registers, `pc = 0`, not halted.
    pub fn new(mem: Vec<Word>) -> Self {
        Self {
            regs: [0; GPR_COUNT],
            pc: 0,
            halted: false,
            mem,
        }
    }

    /// Register file.
    pub fn regs(&self) -> &[u8; GPR_COUNT] {
        &self.regs
    }

    /// Read one GPR. Panics if `i >= 8`.
    pub fn reg(&self, i: usize) -> u8 {
        self.regs[i]
    }

    /// Write one GPR. Panics if `i >= 8`.
    pub fn set_reg(&mut self, i: usize, value: u8) {
        self.regs[i] = value;
    }

    /// Instruction index of the next fetch.
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// True after a `HALT` has executed.
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Fetch, decode, and execute one instruction.
    ///
    /// Already-halted: no-op `Ok`.
    /// `HALT`: set halted and leave `pc` on that instruction.
    /// ALU/`MOV`: write `rd`, then increment `pc`.
    pub fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Ok(());
        }
        let word = *self.mem.get(self.pc).ok_or(Error::PcOutOfRange {
            pc: self.pc,
            len: self.mem.len(),
        })?;
        let inst = decode(word)?;
        match inst.op {
            Opcode::Halt => {
                self.halted = true;
            }
            Opcode::Add => {
                self.write_alu(inst, u8::wrapping_add);
            }
            Opcode::Sub => {
                self.write_alu(inst, u8::wrapping_sub);
            }
            Opcode::And => {
                self.write_alu(inst, |a, b| a & b);
            }
            Opcode::Or => {
                self.write_alu(inst, |a, b| a | b);
            }
            Opcode::Xor => {
                self.write_alu(inst, |a, b| a ^ b);
            }
            Opcode::Mov => {
                let v = self.regs[inst.rs1 as usize];
                self.regs[inst.rd as usize] = v;
                self.pc += 1;
            }
        }
        Ok(())
    }

    fn write_alu(&mut self, inst: Instruction, f: fn(u8, u8) -> u8) {
        let a = self.regs[inst.rs1 as usize];
        let b = self.regs[inst.rs2 as usize];
        self.regs[inst.rd as usize] = f(a, b);
        self.pc += 1;
    }

    /// `step` until `HALT` or `max_steps`. Returns the number of steps taken.
    pub fn run(&mut self, max_steps: usize) -> Result<usize, Error> {
        let mut n = 0;
        while !self.halted && n < max_steps {
            self.step()?;
            n += 1;
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec encoding: opcode 15–12, rd 11–9, rs1 8–6, rs2 5–3.
    fn word(op: u16, rd: u16, rs1: u16, rs2: u16) -> Word {
        (op << 12) | (rd << 9) | (rs1 << 6) | (rs2 << 3)
    }

    fn cpu_with(regs: [(usize, u8); 3], inst: Word) -> Cpu {
        let mut cpu = Cpu::new(vec![inst, word(0, 0, 0, 0)]);
        for (i, v) in regs {
            cpu.set_reg(i, v);
        }
        cpu
    }

    fn exec(regs: [(usize, u8); 3], inst: Word) -> Cpu {
        let mut cpu = cpu_with(regs, inst);
        cpu.step().unwrap();
        cpu
    }

    // --- decode: these fail if the nibble/field map is wrong ---

    #[test]
    fn decode_halt() {
        let inst = decode(0b0000_101_010_001_111).unwrap();
        assert_eq!(inst.op, Opcode::Halt);
        assert_eq!(inst.rd, 0b101);
        assert_eq!(inst.rs1, 0b010);
        assert_eq!(inst.rs2, 0b001);
    }

    #[test]
    fn decode_add_field_positions() {
        // ADD r1, r2, r3  — binary written in the documented field widths
        let w: Word = 0b0001_001_010_011_000;
        assert_eq!(w, 0x1298);
        let inst = decode(w).unwrap();
        assert_eq!(inst.op, Opcode::Add);
        assert_eq!(inst.rd, 1);
        assert_eq!(inst.rs1, 2);
        assert_eq!(inst.rs2, 3);
    }

    #[test]
    fn decode_sub_not_add() {
        let inst = decode(0b0010_000_001_010_000).unwrap();
        assert_eq!(inst.op, Opcode::Sub);
        assert_ne!(inst.op, Opcode::Add);
    }

    #[test]
    fn decode_and_or_xor_opcodes() {
        assert_eq!(decode(0b0011_000_000_000_000).unwrap().op, Opcode::And);
        assert_eq!(decode(0b0100_000_000_000_000).unwrap().op, Opcode::Or);
        assert_eq!(decode(0b0101_000_000_000_000).unwrap().op, Opcode::Xor);
    }

    #[test]
    fn decode_mov_uses_rs1_field() {
        let inst = decode(0b0110_111_100_011_000).unwrap();
        assert_eq!(inst.op, Opcode::Mov);
        assert_eq!(inst.rd, 7);
        assert_eq!(inst.rs1, 4);
        assert_eq!(inst.rs2, 3);
    }

    #[test]
    fn decode_unknown_opcode_7() {
        assert_eq!(decode(0b0111_000_000_000_000), Err(Error::UnknownOpcode(7)));
    }

    #[test]
    fn decode_unknown_opcode_15() {
        assert_eq!(decode(0xF000), Err(Error::UnknownOpcode(0xF)));
    }

    #[test]
    fn decode_ignores_low_three_bits() {
        let a = decode(0b0001_001_010_011_000).unwrap();
        let b = decode(0b0001_001_010_011_111).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn encode_matches_documented_bits() {
        let inst = Instruction {
            op: Opcode::Add,
            rd: 1,
            rs1: 2,
            rs2: 3,
        };
        assert_eq!(encode(inst), 0b0001_001_010_011_000);
        assert_eq!(decode(encode(inst)).unwrap(), inst);
    }

    // --- ALU: numeric oracles; fail if the operator is swapped ---

    #[test]
    fn add_3_plus_5() {
        let cpu = exec([(1, 3), (2, 5), (0, 0xAA)], word(1, 0, 1, 2));
        assert_eq!(cpu.reg(0), 8);
    }

    #[test]
    fn add_wraps_at_256() {
        let cpu = exec([(1, 0xFF), (2, 1), (0, 0)], word(1, 0, 1, 2));
        assert_eq!(cpu.reg(0), 0);
    }

    #[test]
    fn sub_is_not_commutative() {
        let cpu = exec([(1, 10), (2, 3), (0, 0)], word(2, 0, 1, 2));
        assert_eq!(cpu.reg(0), 7);
        let cpu = exec([(1, 10), (2, 3), (0, 0)], word(2, 0, 2, 1));
        assert_eq!(cpu.reg(0), 3u8.wrapping_sub(10));
        assert_eq!(cpu.reg(0), 249);
    }

    #[test]
    fn sub_wraps_underflow() {
        let cpu = exec([(1, 0), (2, 1), (0, 0)], word(2, 0, 1, 2));
        assert_eq!(cpu.reg(0), 0xFF);
    }

    #[test]
    fn and_clears_bits() {
        let cpu = exec(
            [(1, 0b1100_1100), (2, 0b1010_1010), (0, 0)],
            word(3, 0, 1, 2),
        );
        assert_eq!(cpu.reg(0), 0b1000_1000);
    }

    #[test]
    fn or_sets_bits() {
        let cpu = exec(
            [(1, 0b1100_0001), (2, 0b0010_0010), (0, 0)],
            word(4, 0, 1, 2),
        );
        assert_eq!(cpu.reg(0), 0b1110_0011);
    }

    #[test]
    fn xor_flips_bits() {
        let cpu = exec(
            [(1, 0b1111_0000), (2, 0b1010_1010), (0, 0)],
            word(5, 0, 1, 2),
        );
        assert_eq!(cpu.reg(0), 0b0101_1010);
    }

    #[test]
    fn xor_self_is_zero() {
        let cpu = exec([(3, 0x5A), (0, 1), (1, 1)], word(5, 0, 3, 3));
        assert_eq!(cpu.reg(0), 0);
    }

    #[test]
    fn and_is_not_or() {
        let cpu = exec([(1, 0x0F), (2, 0xF0), (0, 0xFF)], word(3, 0, 1, 2));
        assert_eq!(cpu.reg(0), 0);
        assert_ne!(cpu.reg(0), 0xFF);
    }

    #[test]
    fn mov_copies_rs1_ignores_rs2() {
        let cpu = exec([(4, 42), (5, 99), (0, 1)], word(6, 0, 4, 5));
        assert_eq!(cpu.reg(0), 42);
        assert_eq!(cpu.reg(4), 42);
        assert_eq!(cpu.reg(5), 99);
    }

    #[test]
    fn add_into_source_register() {
        let cpu = exec([(1, 2), (2, 3), (3, 0)], word(1, 1, 1, 2));
        assert_eq!(cpu.reg(1), 5);
        assert_eq!(cpu.reg(2), 3);
    }

    // --- machine: halt, PC, isolation, run ---

    #[test]
    fn halt_does_not_execute_following_add() {
        let mut cpu = Cpu::new(vec![word(0, 0, 0, 0), word(1, 0, 1, 2)]);
        cpu.set_reg(1, 1);
        cpu.set_reg(2, 1);
        cpu.set_reg(0, 0);
        let n = cpu.run(16).unwrap();
        assert_eq!(n, 1);
        assert!(cpu.halted());
        assert_eq!(cpu.reg(0), 0);
        assert_eq!(cpu.pc(), 0);
    }

    #[test]
    fn eight_gprs_write_is_isolated() {
        let mut cpu = Cpu::new(vec![word(1, 3, 0, 1)]);
        for i in 0..GPR_COUNT {
            cpu.set_reg(i, i as u8);
        }
        cpu.step().unwrap();
        assert_eq!(cpu.reg(3), 0u8.wrapping_add(1));
        for i in 0..GPR_COUNT {
            if i != 3 {
                assert_eq!(cpu.reg(i), i as u8, "r{i} was clobbered");
            }
        }
    }

    #[test]
    fn pc_advances_on_alu_not_on_second_halt_step() {
        let mut cpu = Cpu::new(vec![word(1, 0, 1, 2), word(0, 0, 0, 0)]);
        cpu.set_reg(1, 1);
        cpu.set_reg(2, 1);
        cpu.step().unwrap();
        assert_eq!(cpu.pc(), 1);
        assert!(!cpu.halted());
        cpu.step().unwrap();
        assert!(cpu.halted());
        assert_eq!(cpu.pc(), 1);
        cpu.step().unwrap();
        assert_eq!(cpu.pc(), 1);
    }

    #[test]
    fn program_add_sub_and() {
        // r1=0b0011_1100, r2=0b0000_1111, r3=0b1111_0000
        // ADD r4, r1, r2 -> 0b0100_1011
        // SUB r5, r4, r2 -> 0b0011_1100
        // AND r0, r5, r3 -> 0b0011_0000
        let mut cpu = Cpu::new(vec![
            word(1, 4, 1, 2),
            word(2, 5, 4, 2),
            word(3, 0, 5, 3),
            word(0, 0, 0, 0),
        ]);
        cpu.set_reg(1, 0b0011_1100);
        cpu.set_reg(2, 0b0000_1111);
        cpu.set_reg(3, 0b1111_0000);
        cpu.run(16).unwrap();
        assert!(cpu.halted());
        assert_eq!(cpu.reg(4), 0b0100_1011);
        assert_eq!(cpu.reg(5), 0b0011_1100);
        assert_eq!(cpu.reg(0), 0b0011_0000);
    }

    #[test]
    fn run_stops_at_max_steps_without_halt() {
        let mut cpu = Cpu::new(vec![word(6, 0, 0, 0), word(6, 0, 0, 0), word(6, 0, 0, 0)]);
        let n = cpu.run(2).unwrap();
        assert_eq!(n, 2);
        assert!(!cpu.halted());
        assert_eq!(cpu.pc(), 2);
    }

    #[test]
    fn pc_out_of_range() {
        let mut cpu = Cpu::new(vec![]);
        assert_eq!(cpu.step(), Err(Error::PcOutOfRange { pc: 0, len: 0 }));
    }

    #[test]
    fn unknown_opcode_does_not_advance_pc() {
        let mut cpu = Cpu::new(vec![0b1000_000_000_000_000]);
        assert_eq!(cpu.step(), Err(Error::UnknownOpcode(8)));
        assert_eq!(cpu.pc(), 0);
        assert!(!cpu.halted());
    }
}
