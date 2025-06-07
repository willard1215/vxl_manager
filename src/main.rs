mod voxel;
use std::io::{self};
use voxel::*;


fn main() -> io::Result<()> {
    let sections = parse_voxel("01749197760947016001.dec_data").unwrap();

    for section in sections {
        for (key, value) in section.entries {
            match value {
            Value::Text(s) => println!("  {} = {}", key, s),
            Value::Binary(data) => println!("  {} : {:?} (binary)", key, data),
            Value::BinaryList(list) => {
                println!("  {} : [", key);
                for (i, entry) in list.iter().enumerate() {
                        println!("    {}: {:?}", i, entry);
                    }
                println!("  ]");
                }
            }
        }

    }

    Ok(())
}
