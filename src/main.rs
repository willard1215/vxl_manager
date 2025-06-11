mod voxel;

use std::{any::type_name_of_val, io::{self}};
use voxel::*;


fn main() -> io::Result<()> {
    
    let sections = parse_section("./test/ca4Block.dec_data")?;
    for section in sections {
        // println!("{}, {}",section.name,type_name_of_val(&section.name));
        match section.name.as_str() {
            "chunk" => {
                parse_chunk_data(&section.data);
            },
            _ => {}
       } 
    }

    Ok(())
}