use std::env;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::{BufReader, BufRead, Write};

fn main() -> Result<(), String>{
    // ./asm file.asm

    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        panic!("usage: {} <input>", args[0]);
    }    
    let input_file_name = &args[1];
    
    let file = File::open(Path::new(input_file_name)).map_err(|err| format!("failed to open file: {}", err))?;

    let mut output: Vec<u8> = Vec::new();
    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        for token in line.split(" ").filter(|x| x.len() > 0) {
            let b = u8::from_str_radix(token, 16).map_err(|err| format!("failed to parse int: {}", err))?;
            output.push(b);
        }
    }

    let mut handle = io::stdout().lock();
    handle.write_all(&output).map_err(|err| format!("failed to write to stdout: {}", err))?;
    return Ok(());
}