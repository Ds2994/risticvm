use std::env;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::{BufReader, BufRead, Write};

use rusticvm::{Instruction, OpCode, Register};

fn parse_numeric(s: &str) -> Result<u8, String> {
    // TODO: handle hex, binary numbers
    if s.len() == 0 {
        return Err("string has no length".to_string());
    }
    let first = s.chars().nth(0).unwrap();
    let (num, radix) = match first {
        '$' => (&s[1..], 16),
        '%' => (&s[1..], 2),
        _ => (s, 10)
    };
    return u8::from_str_radix(num, radix).map_err(|x| format!("{}", x));
}

fn parse_register(s: &str) -> Result<Register, String> {
    return match s {
        "A" => Ok(Register::A),
        _ => Err(format!("unknown register: {}", s)),
    };
}

fn assert_length(parts: &Vec<&str>, n: usize) -> Result<(), String> {
    if parts.len() == n {
        return Ok(());
    } else {
        return Err(format!("expected {} got {}", parts.len(), n));
    }
}

fn handle_line(parts: Vec<&str>) -> Result<Instruction, String> {
    let opcode = OpCode::from_str(parts[0]).ok_or(format!("unkown opcode: {}", parts[0]))?;

    match opcode {
        OpCode::Nop => {
            assert_length(&parts, 1)?;
            return Ok(Instruction::Nop);
        },
        OpCode::Push => {
            assert_length(&parts, 2)?;
            return Ok(Instruction::Push(parse_numeric(parts[1])?));
        },
        OpCode::AddStack => {
            assert_length(&parts, 1)?;
            return Ok(Instruction::AddStack);
        },
        OpCode::PopRegister => {
            assert_length(&parts, 2)?;
            return Ok(Instruction::PopRegister(parse_register(parts[1])?));
        },
        OpCode::PushRegister => {
            assert_length(&parts, 2)?;
            return Ok(Instruction::PushRegister(parse_register(parts[1])?));
        },
        OpCode::Signal => {
            assert_length(&parts, 2)?;
            return Ok(Instruction::Signal(parse_numeric(parts[1])?));
        },
        OpCode::AddRegister => {
            assert_length(&parts, 3)?;
            return Ok(Instruction::AddRegister(
                parse_register(parts[1])?,
                parse_register(parts[2])?
            ));
        }
    }
}

fn main() -> Result<(), String>{
    // ./asm file.asm

    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        panic!("usage: {} <input>", args[0]);
    }    
    let input_file_name = &args[1];
    
    let file = File::open(Path::new(input_file_name)).map_err(|err| format!("failed to open file: {}", err))?;
    let mut output: Vec<u8> = Vec::new();

    // PUSH 10
    // PUSH 8
    // ADDSTACK
    for line in BufReader::new(file).lines() {
        let line_inner = line.map_err(|_x| "foo")?;
        if line_inner.len() == 0 {
            continue;
        }
        if line_inner.chars().nth(0).unwrap() == ';' {
            continue;
        }
        let parts: Vec<_> = line_inner.split(" ").filter(|x| x.len() > 0).collect();
        if parts.len() == 0 {
            continue;
        }
        let instruction = handle_line(parts)?;
        let raw_instruction: u16 = instruction.encode_u16();
        // assumption: "">> 8" needs no mask for u16
        output.push((raw_instruction & 0xff) as u8);
        output.push((raw_instruction >> 8) as u8);
    }

    let mut handle = io::stdout().lock();
    handle.write_all(&output).map_err(|err| format!("failed to write to stdout: {}", err))?;
    return Ok(());
}