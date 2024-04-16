use serde::Deserialize;
use std::io::{stdin, Result};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

#[derive(Deserialize, Debug)]
struct Endpoint {
    socket_address: SocketAddr,
    request: serde_json::Value,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config: Vec<Endpoint> = serde_json::from_reader(stdin())?;
    let mut connections: Vec<JoinHandle<Result<()>>> = Vec::new();
    for endpoint in config {
        connections.push(tokio::spawn(async move {
            let mut conn = TcpStream::connect(endpoint.socket_address).await?;
            let mut u: Vec<u8> = Vec::new();
            let buf = match endpoint.request {
                serde_json::Value::String(s) => {
                    for c in s.as_bytes() {
                        u.push(*c);
                    }
                    u.as_slice()
                }
                serde_json::Value::Array(v) => {
                    for val in v {
                        u.push(match val {
                            serde_json::Value::Number(n) => match n.as_u64() {
                                Some(x) => x as u8,
                                None => panic!("Number doesn't fit into u64"),
                            },
                            _ => panic!("Invalid array value"),
                        })
                    }
                    u.as_slice()
                }
                _ => panic!("Invalid request value"),
            };
            conn.write_all(buf).await?;
            conn.shutdown().await?;
            let mut s = String::new();
            conn.read_to_string(&mut s).await?;
            println!("{}", s);
            Ok(())
        }))
    }
    for connection in connections {
        tokio::try_join!(connection)?.0?;
    }
    Ok(())
}
