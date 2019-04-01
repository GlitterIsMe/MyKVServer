extern crate grpcio;
extern crate protos;

use std::env;
use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};
use protos::kvserver::{Request, Status, OperationType, ResultStatus};
use protos::kvserver_grpc::KvServerClient;

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

    let mut request = Request::new();
    request.set_opt(OperationType::INSERT);
    request.set_key("foo".to_string());
    request.set_value("bar".to_string());

    let check = client.serve(&request).expect("serve failed!");

    let mut request2 = Request::new();
    request2.set_opt(OperationType::GET);
    request2.set_key("foo".to_string());

    let check = client.get_test(&request2).expect("get-test failed!");
}