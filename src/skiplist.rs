mod node;
use node::*;
use random::Source;

pub struct Skiplist{
    head_: Box<Node>, //encapsulate by Box
    max_height_: u16,
    prev_: [Option<Box<Node>>, MAX_HEIGHT()],
    len_: u32,
}

impl Skiplist{
    pub fn new() -> Self{
        Skiplist{
            head_: Box::new(Node{
                data_: (),
                next_: [None; MAX_HEIGHT()]
            }),
            max_height_: 1,
            prev_: [Node; MAX_HEIGHT()],
            len_: 0,
        }
    }

    pub fn insert(&mut self, key: String, value: String){        
        // find the node with largest key and less or euqal than current key
        find_less_or_equal(key);

        // get a random height and construct the Node
        let rand_height = random_height();
        let new_item = KVitem{
            key_len_: key.len(),
            key_: key,
            value_len_: value.len(),
            value_: value,
            type_ = KVType::kKeyValueType;
        };
        let new_node = Box::new(Node{
            data_: new_item,
            next_: [None, MAX_HEIGHT()];
        });

        // insert the node to skiplist
        let i = 0;
        while(i < rand_height){
            new_node.set_next(i, prev_[i]);
            prev_[i].set_next(i, new_node);
            i++;
        }
    }

    pub fn get(&self, key: &str) -> Option<&str>{
        if let Some(x) = find_key(key){
            return x.value();
        }
        None
    }

    pub fn delete(&mut self, key: &str){
        let i = 0;
        let cur = self.head_;
        while(i < MAX_HEIGHT()){
            whlie(cur.next(height) != None && cur.next(height).key() != key){
                cur = cur.next(height);
            }
            if(cur.next(height) != None){
                cur.set_next(height, cur.next(height).next());
            }
            i++;
        }
    }

    pub fn scan(&self, left: &str, right: &str) -> Vec<(&str, &str)>{
        let cur = self.head_;
        let result: Vec[&str] = Vec::new();
        whlie(cur.next(0) != None && cur.next(0).key() < left){
            cur = cur.next(0);
        }
        if cur != None {
            whlie(cur.next(0).key() < right){
                rsult.push_back((cur.key(), cur.value()));
            }
        }
        result;
    }

    fn find_key(&self, key: &str) -> Option<Node>{
        let cur = self.head_;
        let height: i16 = MAX_HEIGHT() as i16;
        while(height >= 0){
            while(cur.next(height) == None || cur.key() < key){
                cur = cur.next(height);
            }
            if(cur.key() == key){
                return cur;
            }
            height--;
        }
        None
    }

    fn find_less_or_equal(&mut self, key: &str){
        let start = self.head_;
        let height:i16 = MAX_HEIGHT() as i16;
        while(height >= 0){
            let cur = self.head_.next(height);
            if cur == None{
                prev_[height] = head_;
            }else{
                while(cur.next(height) !=  None && cur.key() <= key){
                    cur = cur.next(height);
                }
                prev_[height] = cur;
            }
            height--;
        }

    }

    fn random_height() -> u16{
        let mut source = random::default().seed([0, MAX_HEIGHT()]);
        source.read::<u16>()
    }
}