use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use rusticvm::{Machine, Register};

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        panic!("usage: {} <input>", args[0]);
    }    
    let input_file_name = &args[1];
    
    let file = File::open(Path::new(input_file_name)).map_err(|err| format!("failed to open file: {}", err))?;

    let mut reader = BufReader::new(file);
    let mut program: Vec<u8> = Vec::new();
    reader.read_to_end(&mut program).map_err(|err| format!("failed to read file: {}", err))?;

    let mut vm = Machine::new();
    vm.memory.load_from_vec(&program, 0);
    vm.step()?;
    vm.step()?;
    vm.step()?;
    vm.step()?;

    println!("A = {}", vm.get_register(Register::A));
    Ok(())
}