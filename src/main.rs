use std::net::TcpStream;
use std::io::prelude::*;
use std::time::Duration;

mod byte_handling;
pub mod instructions;


// returns new index
fn send_instructions(socket: &mut TcpStream, instruction_buffer: &Vec<u8>, current_idx: usize, ins_buf_size: usize) -> usize {
    // start and end index of slice in instruction_buffer, clamped to length of instruction_buffer
    let base_index = current_idx;
    let target_index = usize::min(instruction_buffer.len(), base_index + ins_buf_size);

    // get the slice
    let instruction_slice = &instruction_buffer[base_index..target_index];

    // debug: println!("Sending instructions... \n\tBase index: {}\n\tTarget index: {}\n\tInstruction slice length: {}", base_index, target_index, instruction_slice.len()); 

    // find the amount of bytes to trim, to shrink it to the next full instruction
    let mut trim_bytes = 0;
    while instruction_slice[target_index - base_index - trim_bytes - 1] != 0x0C {
        trim_bytes += 1;
    }

    // debug: println!("Trimming... \n\tTarget index: {}\n\tVai: {}", target_index - trim_bytes, instruction_slice[target_index - base_index - trim_bytes - 1]); 

    let _ = socket.write(&[0x01]);
    let _ = socket.write_all(&instruction_slice[..instruction_slice.len() - trim_bytes]);
    println!("Sent {} instruction bytes. ({}/{})", instruction_slice.len() - trim_bytes, current_idx + usize::min(ins_buf_size, instruction_buffer.len()) - trim_bytes - 1, instruction_buffer.len());

    current_idx + usize::min(ins_buf_size, instruction_buffer.len()) - trim_bytes - 1
}



fn main() {
    let instruction_pairs: Vec<(i16, i16)> = instructions::load_instructions("./ins.json");
    let instruction_buffer: Vec<u8> = instructions::transform_instructions(&instruction_pairs);
    let mut current_idx: usize = 0;
    let mut ins_buffer_size: u32 = 0;

    let mut stream = TcpStream::connect("192.168.0.16:8180").expect("Error opening stream");
    // stream.set_read_timeout(Some(Duration::new(10, 0)));
    let mut incoming_buffer = [0; 256];

    let _ = stream.write(&[0x00, 0x01]); // greeting byte

    loop {

        let bytes_read = stream.read(&mut incoming_buffer);
        if bytes_read.is_err() || bytes_read.unwrap() == 0 {
            println!("Received nothing!");
            continue;
        }

        if *incoming_buffer.get(0).unwrap() != 0x02 {
            println!("Received a byte for ANY reason.");
        }

        if *incoming_buffer.get(0).unwrap() == 0x01 {
            let protocol_version: u16 = byte_handling::bytes_to_u16(&incoming_buffer, 1);
            ins_buffer_size = byte_handling::bytes_to_u32(&incoming_buffer, 3);
            let max_motor_speed: u32 = byte_handling::bytes_to_u32(&incoming_buffer, 7);
            let min_pulse_width: u32 = byte_handling::bytes_to_u32(&incoming_buffer, 11);
            println!("\nFirmware feedback\n----------------------------- \n |  Drawing Started: {} \n |  Protocol Version: {}\n |  Instruction Buffer Size: {} \n |  Max Motor Speed: {} steps/sec \n |  Min Pulse Width: {} ms", "true", protocol_version, ins_buffer_size, max_motor_speed, min_pulse_width);

            current_idx = send_instructions(&mut stream, &instruction_buffer, current_idx, ins_buffer_size as usize);
        }

        if *incoming_buffer.get(0).unwrap() == 0x03 {
            println!("Received request for more instructions...");
            if current_idx + 1 == instruction_buffer.len() || current_idx > instruction_buffer.len() {
                println!("Out of instructions. Telling firmware drawing has ended.");
                let _ = stream.write(&[0x02]);
                let _ = stream.shutdown(std::net::Shutdown::Both);
                break;
            } else if current_idx < instruction_buffer.len() {
                current_idx = send_instructions(&mut stream, &instruction_buffer, current_idx, ins_buffer_size as usize);
            }
        }


        

    }
    
}
