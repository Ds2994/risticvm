use std::collections::HashMap;
use crate::memory::{LinearMemory, Addressable};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Register {
    A, B, C, M, SP, PC, BP, FLAGS,
}

impl Register {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            x if x == Register::A as u8 => Some(Register::A),
            x if x == Register::B as u8 => Some(Register::B),
            x if x == Register::C as u8 => Some(Register::C),
            x if x == Register::M as u8 => Some(Register::M),
            x if x == Register::SP as u8 => Some(Register::SP),
            x if x == Register::PC as u8 => Some(Register::PC),
            x if x == Register::BP as u8 => Some(Register::BP),
            x if x == Register::FLAGS as u8 => Some(Register::FLAGS),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Push(u8),
    PopRegister(Register),
    AddStack,
    AddRegister(Register, Register),
    Signal(u8),
}

impl Instruction {

    fn encode_r1(r: Register) -> u16 {
        ((r as u16) & 0xf) << 8
    }

    fn encode_r2(r: Register) -> u16 {
        ((r as u16) & 0xf) << 12
    }

    fn encode_num(u: u8) -> u16 {
        (u as u16) << 8
    }

    fn encode_rs(r1: Register, r2: Register) -> u16 {
        Self::encode_r1(r1) | Self::encode_r2(r2)
    }

     pub fn encode_u16(&self) -> u16 {
        match self {
            Self::Nop => OpCode::Nop as u16,
            Self::Push(x) => OpCode::Push as u16 | Self::encode_num(*x),
            Self::PopRegister(r) => OpCode::PopRegister as u16 | Self::encode_r1(*r),
            Self::Signal(s) => OpCode::Signal as u16 | Self::encode_num(*s),
            Self::AddStack => OpCode::AddStack as u16,
            Self::AddRegister(r1, r2) => OpCode::AddRegister as u16 | Self::encode_rs(*r1, *r2),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Nop = 0x0,
    Push = 0x1,
    PopRegister = 0x2,
    Signal = 0x0f, 
    AddStack = 0x10,
    AddRegister = 0x11,
}

impl OpCode {

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Nop" => Some(Self::Nop),
            "Push" => Some(Self::Push),
            "PopRegister" => Some(Self::PopRegister),
            "Signal" => Some(Self::Signal),
            "AddStack" => Some(Self::AddStack),
            "AddRegister" => Some(Self::AddRegister),
            _ => None,
        }
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            x if x == Self::Nop as u8 => Some(Self::Nop),
            x if x == Self::Push as u8=> Some(Self::Push),
            x if x == Self::PopRegister as u8 => Some(Self::PopRegister),
            x if x == Self::Signal as u8 => Some(Self::Signal),
            x if x == Self::AddStack as u8 => Some(Self::AddStack),
            x if x == Self::AddRegister as u8 => Some(Self::AddRegister),
            _ => None,
        }
    }
}

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

    pub fn get_register(&self, r: Register) -> u16 {
        return self.registers[r as usize];
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