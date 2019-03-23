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

use protos::kvserver::{Request, Status};
use protos::kvserver_grpc::{self, KVServer};

#[derive(Clone)]
struct KVServerService{
    kvmap: BTreeMap<String, String>,
    entry_num: u64,
};

impl KVServer for KVServerService{
    fn Serve(&mut self, ctx: RpcContext, request: Request, sink: UnarySink<Status>){
        println!("get a kv item {:?}", request);
        let mut result = Status::new();
        match request.type{
            OperationType::INSERT =>{
                self.kvmap.insert(request.key, request.value);
                self.entry_num++;
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::GET => {
                result.set_status(ResultStatus::kNotFound);
                if let Some(x) = self.kvmap.get(&request.key){
                    result.set_value(String::from(x));// 返回一个String？
                    result.set_status(ResultStatus::kSuccess);
                }
            },

            OperationType::DELETE => {
                result.set_status(ResultStatus::kNotFound);
                if let Some(x) = self.kvmap.remove(&request.key){
                    result.set_status(ResultStatus::kSuccess);
                    self.entry_num--;
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
    let env = Arc::new(Enviroment::new(1));
    let service  = kvserver_grpc::create_kvserver(KVServerService);
    let mut server = ServerBuild::new(env)
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
