use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::task::JoinHandle;

type State = HashMap<String, u64>;
type ListenerStatesHashMap = HashMap<SocketAddr, State>;
type ListenerStatesMutex = Mutex<ListenerStatesHashMap>;
type ListenerStates = Arc<ListenerStatesMutex>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = std::env::args();
    // TODO: global state (all counts)
    let listener_states =
        ListenerStates::new(ListenerStatesMutex::new(ListenerStatesHashMap::new()));
    let mut listeners: Vec<JoinHandle<Result<()>>> = Vec::new();
    for arg in args.skip(1) {
        // TODO: allow passing only port
        let ls = listener_states.clone();
        listeners.push(tokio::spawn(async move {
            let arg = arg.clone();
            let listener = tokio::net::TcpListener::bind(&arg).await?;
            loop {
                let (mut socket, _) = listener.accept().await?;
                let addr: SocketAddr = arg.parse().unwrap();
                let mut string = String::new();
                socket.read_to_string(&mut string).await?;
                println!("{}", string);
                let response = {
                    let mut ls = ls.lock().unwrap();
                    let state = match ls.get_mut(&addr) {
                        Some(map) => map,
                        None => {
                            match ls.insert(addr, HashMap::<String, u64>::new()) {
                                Some(_) => unreachable!(),
                                None => (),
                            };
                            match ls.get_mut(&addr) {
                                Some(map) => map,
                                None => unreachable!(),
                            }
                        }
                    };
                    let segments: Vec<&str> = string.split(' ').collect();
                    let amount: u64 = match segments[0].parse() {
                        Ok(s) => s,
                        Err(e) => return Result::Err(Error::new(ErrorKind::InvalidData, e)),
                    };
                    let animal: String = String::from(segments[1]);
                    let val = match state.get(&animal) {
                        Some(n) => *n,
                        None => 0,
                    };
                    state.insert(animal, val + amount);

                    let response: String = serde_json::to_string(state).unwrap();
                    response
                };
                println!("{}", response);
                socket.write_all(response.as_bytes()).await?;
                socket.shutdown().await?;
            }
        }));
    }
    for listener in listeners {
        tokio::try_join!(listener)?.0?;
    }
    Ok(())
}
