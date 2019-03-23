extern crate futures;
extern crate grpcio;
extern crate protos;

use std::io::Read;
use std::sync::Arc;
use std::{io, thread};
use std::collections::BTreeMap;

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::{self, KvServer};

#[derive(Clone)]
struct KVServerService{
    kvmap: BTreeMap<String, String>,
    entry_num: u64,
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
