extern crate grpcio;
extern crate protos;
extern crate stopwatch;

use std::env;
use std::sync::Arc;
use std::io::{self, Read};

use grpcio::{ChannelBuilder, EnvBuilder};
use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::KvServerClient;

use stopwatch::Stopwatch;


fn main(){
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Expected exactly one argument, the port to connect to.")
    }
    let port = args[1]
        .parse::<u16>()
        .expect(format!("{} is not a valid port number", args[1]).as_str());

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("localhost:{}", port).as_str());
    let client = KvServerClient::new(ch);

    let sw = Stopwatch::start_new();
    println!("system on");
    println!("input operation:");
    loop{
        let mut request = Request::new();
        let mut raw_opt = String::new();
        io::stdin().read_line(&mut raw_opt);
        let raw_opt2 = raw_opt.trim();
        println!("opt-{}", raw_opt);
        let opt: Vec<&str> = raw_opt2.split(' ').collect();
        for elem in &opt{
            println!("{}", elem);
        }
        match opt[0]{
            "put" => {
                request.set_opt(OperationType::INSERT);
                request.set_key(opt[1].to_string());
                request.set_value(opt[2].to_string());
            },
            "get" => {
                request.set_opt(OperationType::GET);
                request.set_key(opt[1].to_string());
            },
            "delete" => {
                request.set_opt(OperationType::DELETE);
                request.set_key(opt[1].to_string());
            },
            "scan" => {
                request.set_opt(OperationType::SCAN);
                request.set_key(opt[1].to_string());
                request.set_value(opt[2].to_string());
            },
            "persist" => {
                request.set_opt(OperationType::PERSIST);
            },
            "shutdown" => break,
            _ => println!("unknow instruction, please enter again"),
        }
        let check = client.serve(&request).expect("serve failed!");
    }

    println!("system down, runs {}ms", sw.elapsed_ms());
}