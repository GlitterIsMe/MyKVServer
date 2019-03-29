extern crate futures;
extern crate grpcio;
extern crate protos;

use std::io::Read;
use std::sync::Arc;
use std::{io, thread};
//use std::collections::BTreeMap;

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::{self, KvServer};

use kvmap::MemIndex;
use basic::DataType;

#[derive(Debug,Clone)]
struct KVServerService{
    kvmap: BTreeMap<String, String>,
    entry_num: u64,
    mem_: Option<Box<MemIndex>>,
    imm_: Option<Box<MemIndex>>,
    need_flush_: bool,
    need_compact_: bool,
}//struct声明后面不要分号

impl KvServer for KVServerService{
    fn serve(&mut self, ctx: RpcContext, request: Request, sink: UnarySink<Status>){
        println!("get a kv item {:?}", request);
        let mut result = Status::new();
        match request.opt{
            OperationType::INSERT =>{
                self.kvmap.insert(request.key, request.value);
                self.entry_num += 1;// Rust没有++和--
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::GET => {
                result.set_status(ResultStatus::kNotFound);
                if let Some(x) = self.kvmap.get(&request.key){
                    result.set_value(x.to_string());// 返回一个String？
                    result.set_status(ResultStatus::kSuccess);
                }
            },

            OperationType::DELETE => {
                result.set_status(ResultStatus::kNotFound);
                if let Some(x) = self.kvmap.remove(&request.key){
                    result.set_status(ResultStatus::kSuccess);
                    self.entry_num -= 1;
                }
            },

            OperationType::SCAN =>{
                //TODO
            },

            _ =>{
                println!("No Matched Operation");
                result.set_status(ResultStatus::kFailed);
            }
        }
    }
}

impl KVServerService{
    //basic operation
    pub fn Put(&mut self, key: String, type: DataType, value：String) -> bool{
        CheckForWriteSpace();
        if let Some(x) = self.mem_{
            x.Put(key, (type, value))
        }
        false
    }

    pub fn Get(&self, key: &str) -> Option(String){
        // get from mem firstly
        if let Some(x) = self.mem_{
            x.Get(key)
        }
        // get from imm secondly
        if let Some(x) = self.imm_{
            x.Get(x)
        }
        // get from file

    }

    pub fn NewIter(&self) -> Iter<String, String>{

    }

    fn MaxBufferSize() -> u64{
        4 * 1024 * 1024
    }

    fn CheckForWriteSpace(&mut self, size: u64){
        // check whether the space in kvmap is enough
        // or build a new kvmap and trigger flush
        if let Some(x) = self.mem_{
            if x.ApproximateUsage() < self.MaxBufferSize(){
                //do nothing
            }else{
                match self.imm_{
                    Some(x) => {
                        // wait util flush finish
                        // wait 
                    },
                    None =>,
                }
                SwitchKVMap();
            }
        }
    }

    fn SwitchKVMap(&mut self){
        // build a new kvmap
        // imm must be None now
        match self.mem_{
            Some(x) => {
                self.imm_ = Some(x);
                self.mem_ = Some(Box::new(MemIndex::new()));
            },
            None => println!("no mem currently");
        }
    }

    fn BackgroundFlush(&mut self){
        // get the imm and write it to file
    }

    fn BackgroundCompaction(&mut self){
        
    }

}


fn main(){
    let env = Arc::new(Environment::new(1));
    let service  = kvserver_grpc::create_kv_server(KVServerService{
        kvmap: BTreeMap::new(),
        entry_num: 0,
    });//结构体在这里初始化，例子中的结构体是个单元结构体所以看起来像是直接用结构体名
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();

    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
    
}
