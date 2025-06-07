use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};

use csv::Reader;

#[derive(Debug)]
pub enum Value {
    Text(String),
    Binary(Vec<u8>),
    BinaryList(Vec<Vec<u8>>),
}


#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub entries: HashMap<String, Value>,
}

pub fn parse_voxel(path: &str) -> io::Result<Vec<Section>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut sections = Vec::new();
    let mut current_section = Section {
        name: String::new(),
        entries: HashMap::new(),
    };

    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    let mut last_key: Option<String> = None;

    while reader.read(&mut byte)? == 1 {
        //1바이트 버퍼
        let b = byte[0];

        //0x0A 플래그 발견 시
        if b == b'\n' {
            //current_section과 line
            process_line(&mut current_section, &line, &mut last_key);
            line.clear();
        } else if b == b'[' {
            // 만약 이전 섹션에 내용이 있다면 저장
            if !current_section.name.is_empty() || !current_section.entries.is_empty() {
                sections.push(current_section);
                current_section = Section {
                    name: String::new(),
                    entries: HashMap::new(),
                };
            }

            // 섹션 이름 파싱
            let mut name_bytes = Vec::new();
            while reader.read(&mut byte)? == 1 {
                let c = byte[0];
                if c == b']' {
                    break;
                }
                name_bytes.push(c);
            }
            current_section.name = String::from_utf8_lossy(&name_bytes).to_string();
            last_key = None;
        } else {
            line.push(b);
        }
    }

    // 마지막 줄 및 섹션 추가
    if !line.is_empty() {
        process_line(&mut current_section, &line, &mut last_key);
    }
    if !current_section.name.is_empty() || !current_section.entries.is_empty() {
        sections.push(current_section);
    }

    Ok(sections)
}

fn process_line(
    section: &mut Section,
    line: &[u8],
    last_key: &mut Option<String>,
) {
    if line.is_empty() {
        return;
    }

    // continuation line for binary list
    if line.starts_with(b"+:") {
        if let Some(key) = last_key {
            let val = line[2..].to_vec();
            match section.entries.get_mut(key) {
                Some(Value::BinaryList(list)) => {
                    list.push(val);
                }
                Some(Value::Binary(prev)) => {
                    // promote to BinaryList
                    let old = std::mem::take(prev);
                    section.entries.insert(
                        key.clone(),
                        Value::BinaryList(vec![old, val]),
                    );
                }
                _ => {
                    // unexpected continuation → ignore or error
                }
            }
        }
        return;
    }

    // text value
    if let Some(pos) = line.iter().position(|&b| b == b'=') {
        let key = String::from_utf8_lossy(&line[..pos]).to_string();
        let val = String::from_utf8_lossy(&line[pos + 1..]).to_string();
        section.entries.insert(key.clone(), Value::Text(val));
        *last_key = Some(key);
    }

    // binary value (start of Binary or BinaryList)
    else if let Some(pos) = line.iter().position(|&b| b == b':') {
        let key = String::from_utf8_lossy(&line[..pos]).to_string();
        let val = line[pos + 1..].to_vec();
        section.entries.insert(key.clone(), Value::Binary(val));
        *last_key = Some(key);
    }
}
