pub const fn MAX_LENGTH() -> u16{
    4
}

//TODO : support other type of type like deletion and so on...
pub enum KVType{
    kKeyValueType,
}

pub struct KVitem{
    key_len_: u32,
    key_: String,
    value_len_: u32,
    value_: String,
    type_: KVType,
    sequence_: u64,
}

#[derive(Clone, Debug)]
pub struct Node{
    data_: KVitem,
    // TODO: fixed height -> dynamic height
    next_: [Option<Box<Node>>, MAX_LENGTH()],
}

impl Node {
    fn new(data: KVitem) -> Self{
        Node{
            data_: data,
            next_: [None; MAX_LENGTH()],
        }
    }

    pub fn set_next(&mut self, height: u16, node: Self){
        self.next_[height] = Some(Box::new(node));// Does Box will get the ownership of Node?
    }

    pub fn next(&self, height: u16) -> Some(Node){
        next_[height]?
    }

    pub fn key() -> &str{
        data_.key_
    }

    pub fn value() -> &str{
        data.value_
    }
}