extern crate futures;
extern crate grpcio;
extern crate protos;

use std::io::Read;
use std::sync::{Arc, Condvar, Mutex};
use std::{io, thread};
use std::collections::BTreeMap;
use::std::sync::atomic::{AtomicBool, Ordering};
use std::iter;

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::{self, KvServer};

mod kvmap;
use self::kvmap::MemIndex;

mod myio;
use self::myio::builder::PersistToFile;

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
    main_buffer_: Option<Box<MemIndex>>,
    // work when the main buffer is been persisted
    sub_buffer_: Option<Box<MemIndex>>,
    // newest version of persist data
    latest_persistent_version_: u64,
    // whether the main buffer is been persisted
    // TODO : use a atomic variable
    persistent_working_: Arc<AtomicBool>,

    system_working_: Arc<AtomicBool>,

    bg_working_: Arc<AtomicBool>,

    system_stall_: Arc<(Mutex<bool>, Condvar)>,

    cv_: Arc<(Mutex<bool>, Condvar)>,

    env_: Env,
}//struct声明后面不要分号

impl KvServer for KVServerService{
    fn serve(&mut self, ctx: RpcContext, request: Request, sink: UnarySink<Status>){
        println!("get a kv item {:?}", request);
        let result = self.ExcuteOpt(&request);
        let f = sink
            .success(result.clone())
            .map(move |_| println!("Responded with result"))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn get_test(&mut self, ctx: RpcContext, request: Request, sink: UnarySink<Status>){
        let mut result = Status::new();
        //println!("map len is {} and {} entries", self.kvmap.len(), self.entry_num);
        //let addr = self as *const Self as usize;
        //println!("self addr is 0x{:X}", addr);
        //result.set_status(ResultStatus::kNotFound);
        /* if let Some(x) = self.kvmap.get(&request.key){
            println!("get {}", x);
            result.set_value(x.to_string());// 返回一个String？
            result.set_status(ResultStatus::kSuccess);
        } */
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
            entry_num_ : 0,
            main_buffer_: Some(Box::new(MemIndex::new())),
            sub_buffer_: Some(Box::new(MemIndex::new())),
            latest_persistent_version_: 1,
            persistent_working_: Arc::new(AtomicBool::new(false)),
            system_working_: Arc::new(AtomicBool::new(true)),
            bg_working_: Arc::new(AtomicBool::new(false)),
            system_stall_: Arc::new((Mutex::new(false), Condvar::new())),
            cv_: Arc::new((Mutex::new(false), Condvar::new())), 
            env_: Env::new(),
        }
    }

    pub fn ExcuteOpt(&mut self, request: &Request) -> Status{
        let mut result = Status::new();
        match request.opt{
            OperationType::INSERT =>{
                //self.entry_num += 1;// Rust没有++和--
                self.Put(request.key, request.value);
                //println!("self addr is 0x{:X}", self as *const Self as usize);
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::GET => {
                result.set_status(ResultStatus::kNotFound);
                if let Some(x) = self.Get(&request.key){
                    result.set_value(x.to_string());// 返回一个String？
                    result.set_status(ResultStatus::kSuccess);
                }
            },

            OperationType::DELETE => {
                self.Delete(&request.key);
                result.set_status(ResultStatus::kSuccess);
            },

            OperationType::SCAN =>{
                //TODO
            },

            OperationType::PERSIST =>{
                self.Persist();
                result.set_status(ResultStatus::kSuccess);
            },

            _ =>{
                println!("No Matched Operation");
                result.set_status(ResultStatus::kFailed);
            }
        }
        result
    }

    pub fn Put(&mut self, key: String, value: String) -> bool{
        let mut buffer_to_insert = None;
        {
            let system_stall_clone = self.system_stall_.clone();
            let &(mu_ref, cv_ref) = &*system_stall_clone;
            let mut stalling = mu_ref.lock().unwrap();
            while *stalling {
                stalling = cv_ref.wait(stalling).unwrap();
            }
            stalling = false;
        }
    
        let persistent_working_clone = self.persistent_working_.clone();
        if !persistent_working_clone.load(Ordering::SeqCst) {
            buffer_to_insert = self.main_buffer_;
        }else{
            buffer_to_insert = self.sub_buffer_;
        }
        // TODO : use mutex to protect buffer under multithread
        if let Some(x) = buffer_to_insert{
            x.Put(key, value);
        }
        false
    }

    pub fn Get(&self, key: &str) -> Option<String>{
        {
            let system_stall_clone = self.system_stall_.clone();
            let &(mu_ref, cv_ref) = &*system_stall_clone;
            let mut stalling = mu_ref.lock().unwrap();
            while *stalling {
                stalling = cv_ref.wait(stalling).unwrap();
            }
            stalling = false;
        }
        // get from mem firstly
        if let Some(x) = self.main_buffer_{
            x.Get(key)
        }
        // get from imm secondly
        if let Some(x) = self.sub_buffer_{
            if x.len() > 0{
                x.Get(key)
            }
        }
    }

    pub fn Delete(&mut self, key: &str){
        {
            let system_stall_clone = self.system_stall_.clone();
            let &(mu_ref, cv_ref) = &*system_stall_clone;
            let mut stalling = mu_ref.lock().unwrap();
            while *stalling {
                stalling = cv_ref.wait(stalling).unwrap();
            }
            stalling = false;
        }

        // delete from mem firstly
        if let Some(x) = self.main_buffer_{
            x.Delete(key)
        }
        // delete from imm secondly
        if let Some(x) = self.sub_buffer_{
            x.Delete(key)
        }
    }

    pub fn NewIter(&self) -> Iterator<String, String>{

    }

    pub fn Persist(&mut self){
        let mut bg_wk_clone = self.bg_working_.clone();
        if !bg_wk_clone.load(Ordering::SeqCst) {
            thread::spawn(|| {
                self.BackgroundThreadEntryPoint(self);
            });
            bg_wk_clone.store(true, Ordering::SeqCst);
        }
        let &(mu_ref, cv_ref) = &*self.cv_.clone();
        let mut start = mu_ref.lock.unwrap();
        *start = true;
        cv_ref.notify_one(); 
    }

    fn BackgroundThreadEntryPoint(server: KVServerService){
        server.BgWorkMain();
    }

    fn BgWorkMain(&mut self){
        let sys_working_clone = self.system_working_.clone();
        let cv_clone = self.cv_.clone();
        while sys_working_clone.load(Ordering::SeqCst) {
            let &(ref_mtx, ref_cv) = &*cv_clone;
            let mut process = ref_mtx.lock().unwrap();
            while !*process {
                //触发process的时候开始执行persistent
                process = ref_cv.wait(process).unwrap();
            }
            self.BGPersist();
            *process = false;
        }
    }

    fn BGPersist(&mut self){
        let system_stall_clone = self.system_stall.clone();
        let persistent_working_clone = self.persistent_working_.clone();
        // handle by background thread
        persistent_working_clone.store(true, Ordering::SeqCst);
        // persist data in main buffer to file in increments
        if let Some(x) = self.main_buffer_{
            PersistToFile(x.NewIter(), self.latest_persistent_version_);
            self.latest_persistent_version_ += 1;
        }
        // move data in subbuffer to main buffer
        // systemt will stall in this period
        if let Some(x) = self.sub_buffer_{
            if x.len() > 0 {
                system_stall_clone.store(true, Ordering::SeqCst);
                if let Some(x) = self.main_buffer_{
                    for (key, value) in x.NewIter(){
                        x.Put(key, value.2);
                    }
                }
                x.Clear();
                system_stall_clone.store(false, Ordering::SeqCst);
            }
        }
        persistent_working_clone.store(false, Ordering::SeqCst);
    }
} 

impl Drop for KVServerService{
    //pub fn Drop(&mut self){
    //impl trait不需要加pub，因为在定义里面已经有了
    fn drop(&mut self){
        let mut working = self.system_working_.clone();
        working.store(false, Ordering::SeqCst);
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
