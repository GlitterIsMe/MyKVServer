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
use std::sync::{Arc,RwLock};
use crate::kvmap::MemIndex;

pub fn persist_to_file(table: Arc<RwLock<MemIndex>>, persist_version: u64){
    // 尝试打开log文件，如果失败则创建文件
    let mut file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open("persisted_data.log").expect("open failed!");
    let mut buffer = String::new();
    for (key, value) in table.write().unwrap().new_iter_mut().filter(|(k,v)|->bool {!v.0}){
        value.0 = true;
        let kvitem = format!("{},{},{},{},{},{},\n", 
        key.len() as u32, 
        &key, 
        true as u8,
        persist_version as u64,
        value.2.len() as u32,
        &value.2);
        buffer.push_str(&kvitem);
        println!("read:{}-{}",key, value.2);
    }
    file.write_all(buffer.as_bytes()).expect("write failed!");
} 