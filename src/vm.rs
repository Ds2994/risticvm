use std::collections::HashMap;
use crate::memory::{LinearMemory, Addressable};
use crate::op::{OpCode, Instruction};
use crate::register::Register;

fn parse_instruction_arg(ins: u16) -> u8 {
    return ((ins & 0xff00) >> 8) as u8;
}

fn parse_instruction(ins: u16) -> Result<Instruction, String> {
    let op = (ins & 0xff) as u8;
    match OpCode::from_u8(op).ok_or(format!("unknown op: {:X}", op))? {
        OpCode::Nop => Ok(Instruction::Nop),
        OpCode::Push => {
            let arg = parse_instruction_arg(ins);
            return Ok(Instruction::Push(arg));
        },
        OpCode::PopRegister => {
            let reg = (ins & 0xf00) >> 8;
            return Register::from_u8(reg as u8)
                .ok_or(format!("unkown register 0x{:X}", reg))
                .map(|r| Instruction::PopRegister(r));
        },
        OpCode::PushRegister => {
            let reg = (ins & 0xf00) >> 8;
            return Register::from_u8(reg as u8)
                .ok_or(format!("unkown register 0x{:X}", reg))
                .map(|r| Instruction::PushRegister(r));
        },
        OpCode::AddStack => {
            return Ok(Instruction::AddStack);
        },
        OpCode::Signal => {
            let arg = parse_instruction_arg(ins);
            return Ok(Instruction::Signal(arg));
        },
        OpCode::AddRegister => {
            let reg1_raw = (ins & 0xf00) >> 8;
            let reg2_raw = (ins & 0xf000) >> 12;
            let reg1 = Register::from_u8(reg1_raw as u8)
                .ok_or(format!("unkown register 0x{:X}", reg1_raw))?;
            let reg2 = Register::from_u8(reg2_raw as u8)
                .ok_or(format!("unkown register 0x{:X}", reg2_raw))?;
            return Ok(Instruction::AddRegister(reg1, reg2));
        }
    }
}

type SignalFunction = fn(&mut Machine) -> Result<(), String>;

pub struct Machine {
    registers: [u16; 8],
    pub halt: bool,
    signal_handlers: HashMap<u8, SignalFunction>,
    pub memory: Box<dyn Addressable>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            halt: false,
            signal_handlers: HashMap::new(),
            memory: Box::new(LinearMemory::new(8*1024)),
        }
    }

    pub fn state(&self) -> String {
        format!("A: {} | B: {} | C: {} | M: {} 
SP: {} | PC: {} | BP: {} 
FLAGS: {:X}
        ",
                self.get_register(Register::A),
                self.get_register(Register::B),
                self.get_register(Register::C),
                self.get_register(Register::M),
                self.get_register(Register::SP),
                self.get_register(Register::PC),
                self.get_register(Register::BP),
                self.get_register(Register::FLAGS),
        )
    }

    pub fn get_register(&self, r: Register) -> u16 {
        return self.registers[r as usize];
    }

    pub fn set_register(&mut self, r: Register, v: u16) {
        self.registers[r as usize] = v;
    }

    pub fn define_handler(&mut self, index: u8, func: SignalFunction) {
        self.signal_handlers.insert(index, func);
    }

    pub fn pop(&mut self) -> Result<u16, String> {
        let sp = self.registers[Register::SP as usize] - 2;
        if let Some(v) = self.memory.read2(sp) {
            self.registers[Register::SP as usize] -= 2;
            return Ok(v);
        } else {
            return Err(format!("memory read fault @ 0x{:X}", sp));
        }
    }

    pub fn push(&mut self, arg: u16) -> Result<(), String> {
        let sp = self.registers[Register::SP as usize];
        if !self.memory.write2(sp, arg) {
            return Err(format!("memory write fault @ 0x{:X}", sp));
        }
        self.registers[Register::SP as usize] += 2;
        return Ok(());
    }

    pub fn step(&mut self) -> Result<(), String> {
        let pc = self.registers[Register::PC as usize];
        let raw_instruction = self.memory.read2(pc).ok_or(format!("PC read failed at 0x{:X}", pc))?;
        self.registers[Register::PC as usize] = pc + 2;

        let instruction = parse_instruction(raw_instruction)?;
        match instruction {
            Instruction::Nop => Ok(()),
            Instruction::Push(arg) => {
                self.push(arg.into())
            },
            Instruction::PopRegister(reg) => {
                let value = self.pop()?;
                self.registers[reg as usize] = value;
                Ok(())
            },
            Instruction::PushRegister(reg) => {
                self.push(self.registers[reg as usize])?;
                Ok(())
            },
            Instruction::AddStack => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a + b)
            },
            Instruction::AddRegister(r1, r2) => {
                self.registers[r1 as usize] += self.registers[r2 as usize];
                Ok(())
            },
            Instruction::Signal(signal) => {
                let sig_fn = self.signal_handlers.get(&signal)
                    .ok_or(format!("unkown signal 0x{:X}", signal))?;
                sig_fn(self)
            },       
        }
    }
}