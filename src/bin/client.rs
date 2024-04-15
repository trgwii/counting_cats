use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(serde::Deserialize, std::fmt::Debug)]
struct Endpoint {
    socket_address: std::net::SocketAddr,
    request: serde_json::Value,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config: Vec<Endpoint> = serde_json::from_reader(std::io::stdin())?;
    let mut connections: Vec<tokio::task::JoinHandle<std::io::Result<()>>> = Vec::new();
    for endpoint in config {
        connections.push(tokio::spawn(async move {
            let mut conn = tokio::net::TcpStream::connect(endpoint.socket_address).await?;
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
            println!("{}", std::str::from_utf8(buf).unwrap());
            conn.write_all(buf).await?;
            let mut s = String::new();
            conn.read_to_string(&mut s).await?;
            println!("{}", s);
            conn.shutdown().await?;
            Ok(())
        }))
    }
    for connection in connections {
        tokio::try_join!(connection)?.0?;
    }
    Ok(())
}
