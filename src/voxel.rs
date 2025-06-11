use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::ops::Deref;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum Value {
    Text(String),
    Binary(Vec<u8>),
    BinaryList(Vec<Vec<u8>>),
}


#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct chunk_coor{
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub fn parse_section(path: &str) -> io::Result<Vec<Section>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut sections = Vec::new();
    let mut current_section = Section {
        name: String::new(),
        data: Vec::new(),
    };

    let mut byte = [0u8; 1];

    while reader.read(&mut byte)? == 1 {
        //1바이트 버퍼
        let b = byte[0];
        let mut name_vector = Vec::new();
        if b == b'[' {
            if !current_section.name.is_empty() || !current_section.data.is_empty() {
                sections.push(current_section);
                current_section = Section {
                    name: String::new(),
                    data: Vec::new(),
                };
            }
            while reader.read(&mut byte)? == 1{
                let c = byte[0];
                if c == b']' {
                    break;
                }
                name_vector.push(c);
            }
            current_section.name = String::from_utf8_lossy(&name_vector).to_string();
        } else {
            current_section.data.push(b);
        }
    }
    Ok(sections)
}

pub fn parse_chunk_data(chunkdata: &Vec<u8>) {
    let mut reader = BufReader::new(chunkdata.deref());
    
    reader.read_u8().unwrap();
    
    let mut version_buffer: [u8; 16] = [0u8; 16];
    
    reader.read_exact(&mut version_buffer).unwrap();

    let version = String::from_utf8_lossy(&version_buffer);

    reader.read_u8().unwrap();
    let mut chunk_start_string: [u8; 5] = [0u8; 5];
    reader.read_exact(&mut chunk_start_string).unwrap();

    while reader.read_u8().unwrap() == b':' {
        //Chunk data length with 4 bytes;
        let len = reader.read_i32::<LittleEndian>().unwrap();

        reader.read_exact(&mut [0u8; 4]).unwrap();

        let chunk_coor = chunk_coor{
            x: reader.read_u8().unwrap(),
            y: reader.read_u8().unwrap(),
            z: reader.read_u8().unwrap(),
        };
        let data_type = reader.read_u8().unwrap();
        // println!("chunk_coor: {:?}",chunk_coor);
        // println!("data_type: {}",data_type);
        let mut left_bytes = len-8;
        match data_type {
            0 => {
                if len == 8 {
                    // print!("type 0 passed");
                } else {
                    panic!("Invalid chunk data length: chunk type 0");
                }
            },
            8 => {
                let mut buffer = vec![0u8; left_bytes as usize];
                reader.read_exact(&mut buffer).unwrap();
                // println!("type 8 passed")
                if chunk_coor.x == 0 || chunk_coor.y == 0 || chunk_coor.z == 1 {
                    for byte in &buffer {
                        print!("{:b}",byte);
                    }
                    println!("\n");
                    for byte in &buffer {
                        print!("{:X}",byte);
                    }
                }
            },
            9 => {
                while left_bytes > 0 {
                    let block_count = reader.read_i16::<LittleEndian>().unwrap();
                    let block_details = reader.read_i16::<LittleEndian>().unwrap();
                    let block_rotate = ((block_details >> 12) & 0xF) as i8;
                    let block_index = (block_details & 0xFFF) as i32;
                    // println!("Block: {}, {}, {}, {}", block_count, block_details, block_rotate, block_index);
                    left_bytes -= 4;
                }
            }
            _ => {panic!("Invalid chunk data type: {}", data_type)}
        }
        if reader.read_u8().unwrap() != b'\n' {panic!("broken chunk");};
        match reader.read_u8() {
            Ok(byte) => {
                if byte != b'+' {
                    panic!("broken chunk");
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                panic!("Error reading chunk: {}", e);
            }
        }
    }
}