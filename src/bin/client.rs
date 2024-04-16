use serde::Deserialize;
use serde_json::Value;
use std::io::{stdin, Result};
use std::net::SocketAddr;
use tokio::io::{stdout, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

#[derive(Deserialize, Debug)]
struct Endpoint {
    socket_address: SocketAddr,
    request: serde_json::Value,
}

fn json_value_to_vec_of_u8(value: Value) -> Option<Vec<u8>> {
    match value {
        Value::String(s) => Some(s.into()),
        Value::Array(arr) => {
            let mut ret = Vec::new();
            for val in arr {
                if let Value::Number(n) = val {
                    if let Some(x) = n.as_u64() {
                        if x < 256 {
                            ret.push(x as u8);
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            Some(ret)
        }
        _ => None,
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config: Vec<Endpoint> = serde_json::from_reader(stdin())?;
    let mut connections: Vec<JoinHandle<Result<()>>> = Vec::with_capacity(config.len());
    for endpoint in config {
        connections.push(tokio::spawn(async move {
            let mut conn = TcpStream::connect(endpoint.socket_address).await?;
            if let Some(v) = json_value_to_vec_of_u8(endpoint.request) {
                conn.write_all(v.as_slice()).await?;
                conn.shutdown().await?;
                let mut s = String::new();
                conn.read_to_string(&mut s).await?;
                s.push('\n');
                stdout().write_all(s.as_bytes()).await?;
            }
            Ok(())
        }))
    }
    for connection in connections {
        connection.await??;
    }
    Ok(())
}
