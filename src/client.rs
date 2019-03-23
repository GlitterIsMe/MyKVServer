extern crate grpcio;
extern crate protos;

use std::env;
use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};
use protos::kvserver::{Request, Status};
use protos::kvserver_grpc::KVServerClient;

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
    let client = KVServerClient::new(ch);

    let mut request = Request::new();
    request.set_type(OperationType::INSERT);
    request.set_key("foo");

    let check = client.Serve(&request).expect("RPC Failed!");
}