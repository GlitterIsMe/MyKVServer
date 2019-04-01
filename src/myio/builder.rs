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
use std::fs::OpenOptions;
use std::iter;

pub fn PersistToFile(iter: &Iterator<String, (bool, u64, String)>, persist_version: u64){
    // 尝试打开log文件，如果失败则创建文件
    let mut file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open("persisted_data.log")?;
    let mut buffer = String::new();
    for (key, value) in iter.filter(|&(k,v)| !v.0){
        let mut kvitem = format!("{},{},{},{},{},{},", 
        key.len() as u32, 
        &key, 
        true as u8,
        persist_version as u64,
        value.2.len() as u32,
        &value.2);
        buffer.push_str(&kvitem);
    }
    file.write(buffer)?;
} 