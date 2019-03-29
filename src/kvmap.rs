use std::collections::BtreeMap;
use basic::DataType;

#[derive(Debug, Clone)]
pub struct MemIndex{
    kvmap_: BTreeMap<String, (DataType, String)>,
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

    pub fn ApproximateUsage(&self) -> u64{
        self.data_memory_usage_
    }

    pub fn EntryNum(&self) -> u64{
        self.entry_num_
    }

    //TODO: lock for concurrent
    pub fn Put(&mut self, key: String, type: DataType, value: String) -> bool{
        let key_len = key.len();
        let value_len = value.len();
        match self.kvmap_.insert(key, (type, value)){
            Some(x) => {
                data_memory_usage_ -= x.len();
                data_memory_usage_ += value_len;
            },
            None => {
                data_memory_usage_ += (key_len + value_len);
                self.entry_num_ += 1;
            }
        }// match/if let/while let后面都没有分号
        true
    }

    //TODO: lock for concurrent
    pub fn Get(&self, key: &str) -> Option(String){
        if let Some(x) = self.kvmap_.get(key){
            match x.0 {
                kValueType => Some(x.1.clone()),
                kDeletionType => None,
            }
        }
        None
    }

    pub fn NewIter(&self) -> Iter<String, String>{
        self.kvmap_.iter();
    }
}