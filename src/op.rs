use crate::register::Register;

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

pub fn parse_instruction(ins: u16) -> Result<Instruction, String> {
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
