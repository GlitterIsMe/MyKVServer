use std::collections::BTreeMap;
use std::collections::btree_map::{Iter, IterMut};
use std::sync::RwLock;
use std::iter;

#[derive(Debug, Clone)]
pub struct MemIndex{
    kvmap_: BTreeMap<String, (bool, u64, String)>,
    entry_num_: u64,
    data_memory_usage_: u64,
}

impl MemIndex{
    pub fn new() -> Self{
        MemIndex{
            kvmap_: BTreeMap::new(),
            entry_num_: 0,
            data_memory_usage_: 0,
        }
    }

    pub fn approximate_usage(&self) -> u64{
        self.data_memory_usage_
    }

    pub fn entry_num(&self) -> u64{
        self.entry_num_
    }

    //TODO: lock for concurrent
    pub fn put(&mut self, key: &str, value: &str) -> bool{

        let key_len = key.len();
        let value_len = value.len();
        match self.kvmap_.insert(key.to_string(), (false, 0, value.to_string())){
            Some(x) => {
                self.data_memory_usage_ -= x.2.len() as u64;
                self.data_memory_usage_ += value_len as u64;
            },
            None => {
                self.data_memory_usage_ += key_len as u64 + value_len as u64;
                self.entry_num_ += 1;
            }
        }// match/if let/while let后面都没有分号
        true
    }

    //TODO: lock for concurrent
    pub fn get(&self, key: &str) -> Option<String>{
        if let Some(x) = self.kvmap_.get(key){
            return Some(x.2.clone());
        }
        None
    }

    pub fn delete(&mut self, key: &str){
        self.kvmap_.remove(key);
    }

    pub fn new_iter(&self) -> Iter<String, (bool, u64, String)>{
        self.kvmap_.iter()
    } 

    pub fn new_iter_mut(&mut self) -> IterMut<String, (bool, u64, String)>{
        self.kvmap_.iter_mut()
    } 

    pub fn clear(&mut self){
        self.kvmap_.clear();
    }
}