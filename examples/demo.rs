//! Run a four-instruction program: ADD, SUB, AND, HALT.
//!
//!   cargo run --example demo

use fromthetransistor::{encode, Cpu, Instruction, Opcode};

fn word(op: Opcode, rd: u8, rs1: u8, rs2: u8) -> u16 {
    encode(Instruction { op, rd, rs1, rs2 })
}

fn main() {
    // r1=0x3C, r2=0x0F, r3=0xF0
    // ADD r4, r1, r2 -> 0x4B
    // SUB r5, r4, r2 -> 0x3C
    // AND r0, r5, r3 -> 0x30
    let mut cpu = Cpu::new(vec![
        word(Opcode::Add, 4, 1, 2),
        word(Opcode::Sub, 5, 4, 2),
        word(Opcode::And, 0, 5, 3),
        word(Opcode::Halt, 0, 0, 0),
    ]);
    cpu.set_reg(1, 0x3C);
    cpu.set_reg(2, 0x0F);
    cpu.set_reg(3, 0xF0);

    let steps = cpu.run(16).expect("demo program");
    println!("steps={steps} halted={} pc={}", cpu.halted(), cpu.pc());
    for i in 0..8 {
        println!("r{i}={:#04x}", cpu.reg(i));
    }
    assert_eq!(cpu.reg(0), 0x30);
    assert!(cpu.halted());
    println!("ok: r0 == 0x30");
}
