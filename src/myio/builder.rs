// use kv item in kvmap to build a DataFile
/**
 * |key_size|key|type|value_size|value|
 * |..................................|
 * |    key     | offset   |   size   |
 * |..................................|
 * |          total entry num         | 
 */
use std::io::prelude::*;
use std::fs::File;

use super::basic::DataType;

pub fn BuildTable(iter: &Iter<String, (DataType, String)>, filename: &str){
    let mut opened_file = File::create(filename)?;
    let mut buffer = String::new();
    let mut meta = String::new();
    let mut off = 0;
    let mut count = 0;
    for (key, value) in iter{
        let size = key.len() + value.1.len() + 1 + 4 * 2;
        let mut kvitem = format!("{}{}{}{}{}", 
        key.len() as u32, 
        key, 
        value.0 as u8,
        value.1.len() as u32,
        value.1);
        buffer.psh_str(&kvitem);

        let mut entry_meta = format!("{}{}{}", key, off, size);
        meta.push_str(&entry_meta);
        count += 1;
    }
    buffer.push_str(&meta);
    buffer.push_str(&format!("{}", count));
    opened_file.write(buffer)?;
}