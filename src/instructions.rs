use std::fs;
use crate::byte_handling;


fn load_file_str(path: &str) -> String {
    fs::read_to_string(path).expect("Error reading file. Does it exist?")
}

pub fn load_instructions(path: &str) -> Vec<(i16, i16)> {
    let deserialized_instructions: Vec<(i16, i16)> = serde_json::from_str(&load_file_str(path).to_string()).expect("Failed to parse values");

    return deserialized_instructions;
}

pub fn transform_instructions(ins: &Vec<(i16, i16)>) -> Vec<u8> {
    let mut bin_ins: Vec<u8> = vec!();
    
    for (lm, rm) in ins {
        let (b1, b2) = byte_handling::i16_to_bytes(*lm);
        bin_ins.push(b1);
        bin_ins.push(b2);

        let (b1, b2) = byte_handling::i16_to_bytes(*rm);
        bin_ins.push(b1);
        bin_ins.push(b2);

        bin_ins.push(0x0C);
    }

    bin_ins
}
