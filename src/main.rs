use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use clap::Parser;
use message::MessageBroadcast;

use protobuf::Message;
use zmq::SocketType::{PUB, SUB};

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

/// ZMQ test program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Remote address, if no address act as server
    #[arg(index = 1, value_name = "ADDRESS")]
    address: Option<String>,
    /// Server port
    #[arg(short, long, value_name = "PORT", default_value_t = 9800)]
    port: u32,
}

fn start_server(port: u32) -> Result<(), zmq::Error> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(PUB)?;
    let bind_address = format!("tcp://0.0.0.0:{port}");
    socket.bind(&bind_address)?;

    println!("Setting up server at:[{}]", &bind_address);

    print!("Enter :");
    let mut input = String::new();

    loop {
        input.clear();

        let Ok(_) = io::stdin().read_line(&mut input) else {
            println!("Read Line failed");
            break;
        };

        let input = input.trim();
        if input.is_empty() {
            break;
        }

        let mut proto = MessageBroadcast::new();
        proto.sender = "zmq_srv".to_string();
        proto.message = input.to_string();
        proto.time = if let Ok(epoch) = SystemTime::now().duration_since(UNIX_EPOCH) {
            epoch.as_secs()
        } else {
            0
        };

        println!("Sending:[{}]", &input);
        match proto.write_to_bytes() {
            Ok(msg) => socket.send(msg, 0)?,
            Err(e) => println!("Error Serializeing message {}", e),
        }

        // std::thread::sleep(Duration::from_secs(10));
    }
    Ok(())
}

fn start_client(address: String, port: u32) -> Result<(), zmq::Error> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(SUB)?;
    let sock_address = format!("tcp://{address}:{port}");

    println!("Connecting to [{}]", &sock_address);
    socket.connect(&sock_address)?;
    socket.set_subscribe(b"")?;

    loop {
        let bytes = socket.recv_msg(0)?;
        let mut proto = MessageBroadcast::new();
        match proto.merge_from_bytes(&bytes) {
            Ok(_) => {
                let date = match DateTime::from_timestamp(proto.time as i64, 0) {
                    Some(dt) => format!("{}", dt),
                    None => "Bad Time".to_string(),
                };
                println!("Received: {}", date);
                println!("Received: {}", proto.sender);
                println!("Received: {}", proto.message);
            }
            Err(e) => println!("Deserialize error {}", e),
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Some(address) = args.address {
        match start_client(address, args.port) {
            Ok(_) => println!("Client finieshed OK"),
            Err(e) => println!("Client error {}", e),
        }
    } else {
        match start_server(args.port) {
            Ok(_) => println!("Server finieshed OK"),
            Err(e) => println!("Server error {}", e),
        }
    }
}
