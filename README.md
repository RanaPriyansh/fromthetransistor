# fromthetransistor

One educational slice: a tiny CPU emulator in Rust.

This is **not** [geohot/fromthetransistor](https://github.com/geohot/fromthetransistor). That outline is a 12-week path from FPGA LUTs through an ARM7 core, a compiler, an OS, a TCP stack, and a text browser. This repository implements none of that.

It is also **not** a tape-out, a synthesizable core, Verilog, an FPGA bitstream, a GPU, or a shipped SoC. It is a software emulator of a toy ISA, with tests.

## What it is

- 8 general-purpose registers (`r0`–`r7`), 8-bit, wrapping arithmetic
- Seven ops: `ADD`, `SUB`, `AND`, `OR`, `XOR`, `MOV`, `HALT`
- Fetch → decode → execute in `Cpu::step`
- Proof: `cargo test` (27 tests) and `cargo run --example demo`

## ISA

16-bit instruction word:

```
15          12 11        9 8         6 5         3 2        0
+-------------+-----------+-----------+-----------+----------+
|   opcode    |    rd     |    rs1    |    rs2    | ignored  |
+-------------+-----------+-----------+-----------+----------+
```

| opcode | mnemonic | effect |
|--------|----------|--------|
| `0000` | `HALT` | stop; `rd`/`rs1`/`rs2` ignored |
| `0001` | `ADD` | `rd = rs1 wrapping+ rs2` |
| `0010` | `SUB` | `rd = rs1 wrapping− rs2` |
| `0011` | `AND` | `rd = rs1 & rs2` |
| `0100` | `OR` | `rd = rs1 \| rs2` |
| `0101` | `XOR` | `rd = rs1 ^ rs2` |
| `0110` | `MOV` | `rd = rs1` (`rs2` ignored) |

Any other opcode is an error. There is no immediate encoding, no memory operand, no flags, no pipeline.

Load values with `Cpu::set_reg`, then run instruction words.

## Build and test

```bash
cargo test
cargo run --example demo
```

CI (GitHub Actions) runs both on every push and pull request.

## License

MIT. See [LICENSE](LICENSE).
