extern crate futures;
extern crate grpcio;
extern crate protos;

use std::io::Read;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::{io, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::iter;
use std::collections::VecDeque;

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::{self, KvServer};

mod kvmap;
use self::kvmap::MemIndex;

mod myio;
use self::myio::builder::persist_to_file;

mod env;
use self::env::Env;
//use basic::DataType;

#[derive(Debug,Clone)]
struct KVServerService{
    // in memory index of kv items
    //kvmap_: BTreeMap<String, String>,
    // total number of kv entries
    //entry_num_: u64,
    // in memory index of kv items
    main_buffer_: Arc<RwLock<MemIndex>>,
    // work when the main buffer is been persisted
    sub_buffer_: Arc<RwLock<MemIndex>>,
    // newest version of persist data
    latest_persistent_version_: u64,
    // whether the main buffer is been persisted
    // TODO : use a atomic variable
    persistent_working_: Arc<AtomicBool>,

    system_working_: Arc<AtomicBool>,

    bg_working_: Arc<AtomicBool>,

    system_stall_: Arc<AtomicBool>,

    cv_: Arc<(Mutex<bool>, Condvar)>,

    bg_task_queue_: Arc<Mutex<VecDeque<(fn(Arc<RwLock<MemIndex>>, u64),Arc<RwLock<MemIndex>>, u64)>>>,

    env_: Env,
}//struct声明后面不要分号

impl KvServer for KVServerService{
    fn serve(&mut self, ctx: RpcContext, request: Request, sink: UnarySink<Status>){
        println!("get a kv item {:?}", request);
        let result = self.excute_opt(&request);
        let f = sink
            .success(result.clone())
            .map(move |_| println!("Responded with result"))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
}

 impl KVServerService{
    //basic operation

    pub fn new() -> KVServerService{
        KVServerService{
            //entry_num_ : 0,
            main_buffer_: Arc::new(RwLock::new(MemIndex::new())),
            sub_buffer_: Arc::new(RwLock::new(MemIndex::new())),
            latest_persistent_version_: 1,
            persistent_working_: Arc::new(AtomicBool::new(false)),
            system_working_: Arc::new(AtomicBool::new(true)),
            bg_working_: Arc::new(AtomicBool::new(false)),
            system_stall_: Arc::new(AtomicBool::new(false)),
            cv_: Arc::new((Mutex::new(false), Condvar::new())), 
            bg_task_queue_: Arc::new(Mutex::new(VecDeque::new())),
            env_: Env::new(),
        }
    }

    pub fn excute_opt(&mut self, request: &Request) -> Status{
        let mut result = Status::new();
        match request.opt{
            OperationType::INSERT =>{
                //self.entry_num += 1;// Rust没有++和--
                // request本身是borrow的，这里所有权不能转移
                self.put(&request.key, &request.value);
                //println!("self addr is 0x{:X}", self as *const Self as usize);
                println!("put [{}-{}]", &request.key, &request.value);
                println!("entry num {}", self.main_buffer_.read().unwrap().entry_num());
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::GET => {
                result.set_status(ResultStatus::kNotFound);
                match self.get(&request.key){
                    Some(x) =>{
                        println!("get [{}-{}]", &request.key, &x);
                        result.set_value(x.to_string());// 返回一个String？
                        result.set_status(ResultStatus::kSuccess);
                    },
                    None =>{
                        println!("Not Found");
                    },
                }
            },

            OperationType::DELETE => {
                self.delete(&request.key);
                println!("entry num {}", self.main_buffer_.read().unwrap().entry_num());
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::SCAN =>{
                //TODO
            },

            OperationType::PERSIST =>{
                println!("persist call");
                self.persist();
                result.set_status(ResultStatus::kSuccess);
            },

            _ =>{
                println!("No Matched Operation");
                result.set_status(ResultStatus::kFailed);
            }
        }
        result
    }

    pub fn put(&mut self, key: &str, value: &str) -> bool{
        {
            let stalling = self.system_stall_.clone();
            while stalling.load(Ordering::SeqCst){/*wait*/}
        }
    
        let persistent_working_clone = self.persistent_working_.clone();
        if !persistent_working_clone.load(Ordering::SeqCst) {
            self.main_buffer_.write().unwrap().put(key, value);
        }else{
            self.sub_buffer_.write().unwrap().put(key, value);
        }
        true
    }

    pub fn get(&self, key: &str) -> Option<String>{
        {
            let stalling = self.system_stall_.clone();
            while stalling.load(Ordering::SeqCst){/*wait*/}
        }
        // get from main firstly
        if let Some(x) = self.main_buffer_.read().unwrap().get(key){
                return Some(x);
            }
        if self.sub_buffer_.read().unwrap().entry_num() != 0{
            // get from sub secondly
            if let Some(x) = self.sub_buffer_.read().unwrap().get(key){
                return Some(x);
            }
        }
        None
    }

    pub fn delete(&mut self, key: &str){
        {
            let stalling = self.system_stall_.clone();
            while stalling.load(Ordering::SeqCst){/*wait*/}
        }
        // delete from mem firstly
        self.main_buffer_.write().unwrap().delete(key);
        if self.sub_buffer_.read().unwrap().entry_num() != 0{
            // delete from sub secondly
            self.sub_buffer_.write().unwrap().delete(key);
        }
    }

    pub fn new_iter(&self){
       
    }

    pub fn persist(&mut self){
        let mut bg_wk_clone = self.bg_working_.clone();
        if !bg_wk_clone.load(Ordering::SeqCst) {
            let sys_working_clone = self.system_working_.clone();
            println!("sysworking {}", sys_working_clone.load(Ordering::SeqCst));
            let cv_clone = self.cv_.clone();
            let queue_clone = self.bg_task_queue_.clone();
            let persistent_working_clone = self.persistent_working_.clone();
            let main_buf = self.main_buffer_.clone();
            let sub_buf = self.sub_buffer_.clone();
            let start_stall = self.system_stall_.clone();
            thread::spawn(move| | {
                println!("spawn persist thread");
                loop{
                    let &(ref mu, ref cv) = &*cv_clone;
                    let mut process = mu.lock().unwrap();
                    while !*process {
                        //触发process的时候开始执行persistent
                        println!("wait for awaking");
                        process = cv.wait(process).unwrap();
                    }
                    println!("persist process");
                    if let Some(x) = queue_clone.lock().unwrap().pop_front(){
                        let (func, index, ver) = x;
                        persistent_working_clone.store(true, Ordering::SeqCst);
                        // persist data in main buffer to file in increments
                        func(index, ver);
                        // move data in subbuffer to main buffer
                        // systemt will stall in this period
                        if sub_buf.read().unwrap().entry_num() > 0 {
                            start_stall.store(true, Ordering::SeqCst);
                            for (key, value) in main_buf.read().unwrap().new_iter(){
                                main_buf.write().unwrap().put(&key, &value.2);
                            }
                            sub_buf.write().unwrap().clear();
                            start_stall.store(false, Ordering::SeqCst);            
                        }
                        persistent_working_clone.store(false, Ordering::SeqCst);
                    }
                    *process = false;
                    if !sys_working_clone.load(Ordering::SeqCst){
                        println!("exit persist thread");
                        break;
                    }
                }
            });
            bg_wk_clone.store(true, Ordering::SeqCst);
        }
        let queue_clone_main = self.bg_task_queue_.clone();
        queue_clone_main.lock().unwrap().push_back((persist_to_file, self.main_buffer_.clone(), self.latest_persistent_version_));
        self.latest_persistent_version_ += 1;
        let &(ref mu, ref cv) = &*self.cv_.clone();
        let mut start = mu.lock().unwrap();
        *start = true;
        println!("persist triggr");
        cv.notify_one(); 
    }
 }

impl Drop for KVServerService{
    //pub fn Drop(&mut self){
    //impl trait不需要加pub，因为在定义里面已经有了
    fn drop(&mut self){
        println!("shutdown and drop");
        //let mut working = self.system_working_.clone();
        //working.store(false, Ordering::SeqCst);
    } 
}


fn main(){
    let env = Arc::new(Environment::new(1));
    let service  = kvserver_grpc::create_kv_server(KVServerService::new());//结构体在这里初始化，例子中的结构体是个单元结构体所以看起来像是直接用结构体名
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
