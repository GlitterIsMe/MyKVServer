use std::collections::BTreeMap;
use std::sync::RwLock;
use std::iter;

#[derive(Debug, Clone)]
pub struct MemIndex{
    kvmap_: RwLock<BTreeMap<String, (bool, u64, String)>>,
    entry_num_: RwLock<u64>,
    data_memory_usage_: RwLock<u64>,
}

impl MemIndex{
    pub fn new() -> Self{
        MemIndex{
            kvmap_: RwLock::new(BTreeMap::new()),
            entry_num_: RwLock::new(0),
            data_memory_usage_: RwLock::new(0),
        }
    }

    pub fn ApproximateUsage(&self) -> u64{
        *self.data_memory_usage_.read().unwrap()
    }

    pub fn EntryNum(&self) -> u64{
        *self.entry_num_.read().unwrap()
    }

    //TODO: lock for concurrent
    pub fn Put(&mut self, key: String, value: String) -> bool{

        let key_len = key.len();
        let value_len = value.len();
        let mut index = self.kvmap_.write().unwrap();
        match *index.insert(key, (false, 0, value)){
            Some(x) => {
                {
                    let mut usage = self.data_memory_usage_.write().unwrap(); 
                    *usage -= x.len();
                    *usage += value_len;
                }
            },
            None => {
                {
                    let mut usage = self.data_memory_usage_.write().unwrap(); 
                    *usage += (key_len + value_len);
                }

                {
                    let mut entry = self.entry_num_.write().unwrap();
                    self.entry += 1;
                }
            }
        }// match/if let/while let后面都没有分号
        true
    }

    //TODO: lock for concurrent
    pub fn Get(&self, key: &str) -> Option<String>{
        let index = self.kvmap_.read().unwrap();
        if let Some(x) = *index.get(key){
            match x.0 {
                kValueType => Some(x.1.clone()),
                kDeletionType => None,
            }
        }
        None
    }

    pub fn Delete(&mut self, key: &str){
        let mut index = self.kvmap_.write().unwrap();
        index.remove(key);
    }

    pub fn NewIter(&self) -> Iterator<String, String>{
        self.kvmap_.read().unwrap().iter()
    }

    pub fn Clear(){
        self.kvmap_.write().unwrap().clear();
    }
}