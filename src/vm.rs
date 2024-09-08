use crate::memory::{LinearMemory, Addressable};

#[derive(Debug)]
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

#[repr(u8)]
#[derive(Debug)]
pub enum Op {
    Nop,
    Push(u8),
    PopRegister(Register),
    AddStack,
    AddRegister(Register, Register),
}

impl Op {
    pub fn value(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

fn parse_instruction(ins: u16) -> Result<Op, String> {
    let op = (ins & 0xff) as u8;
    match op {
        x if x == Op::Nop.value() => Ok(Op::Nop),
        x if x == Op::Push(0).value() => {
            let arg = (ins & 0xff00) >> 8;
            return Ok(Op::Push(arg as u8));
        },
        x if x == Op::PopRegister(Register::A).value() => {
            let reg = (ins & 0xf00) >> 8;
            if let Some(r) = Register::from_u8(reg as u8) {
                return Ok(Op::PopRegister(r));
            } else {
                Err(format!("unkown register 0x{:x}", reg))
            }
        },
        x if x == Op::AddStack.value() => {
            return Ok(Op::AddStack);
        },
        _ => Err(format!("unknown operator 0x{:x}", op)),
    }
}

pub struct Machine {
    registers: [u16; 8],
    pub memory: Box<dyn Addressable>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            memory: Box::new(LinearMemory::new(8*1024)),
        }
    }

    pub fn get_register(&self, r: Register) -> u16 {
        return self.registers[r as usize];
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
        let instruction = self.memory.read2(pc).unwrap();
        self.registers[Register::PC as usize] = pc + 2;

        let op = parse_instruction(instruction)?;
        match op {
            Op::Nop => Ok(()),
            Op::Push(arg) => {
                self.push(arg.into())
            },
            Op::PopRegister(reg) => {
                let value = self.pop()?;
                self.registers[reg as usize] = value;
                Ok(())
            },
            Op::AddStack => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a + b)
            },
            Op::AddRegister(r1, r2) => {
                self.registers[r1 as usize] += self.registers[r2 as usize];
                Ok(())
            },
        }
    }
}