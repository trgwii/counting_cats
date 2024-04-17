use std::collections::HashMap;
use std::env::args;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::ParseIntError;
use std::ops::Sub;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use tokio::io::BufWriter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

type State = HashMap<String, u64>;
struct GlobalState {
    changed: bool,
    last_written: SystemTime,
    state: State,
}
type ListenerStatesHashMap = HashMap<SocketAddr, State>;
type ListenerStatesMutex = Mutex<ListenerStatesHashMap>;
type ListenerStates = Arc<ListenerStatesMutex>;

fn create_listener_states() -> ListenerStates {
    ListenerStates::new(ListenerStatesMutex::new(ListenerStatesHashMap::new()))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = args();
    let listener_states = create_listener_states();
    let global_state = Arc::new(Mutex::new(GlobalState {
        changed: false,
        last_written: SystemTime::UNIX_EPOCH,
        state: State::new(),
    }));
    let global_state_file_writer = global_state.clone();
    let mut listeners: Vec<JoinHandle<io::Result<()>>> = Vec::new();
    for arg in args.skip(1) {
        let ls = listener_states.clone();
        let gs = global_state.clone();
        let port_result: Result<u16, ParseIntError> = arg.clone().parse();
        if let Err(_) = port_result {
            continue;
        }
        listeners.push(tokio::spawn(async move {
            let addr =
                SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port_result.unwrap());
            let listener = tokio::net::TcpListener::bind(addr).await?;
            loop {
                let (mut socket, _) = listener.accept().await?;
                let ls = ls.clone();
                let gs = gs.clone();
                tokio::spawn(async move {
                    let (mut read_socket, write_socket) = socket.split();
                    let mut bw = BufWriter::new(write_socket);
                    let mut string = String::new();
                    read_socket.read_to_string(&mut string).await.unwrap();
                    let response = {
                        let mut ls = ls.lock().unwrap();
                        let mut gs = gs.lock().unwrap();
                        let state = match ls.get_mut(&addr) {
                            Some(map) => map,
                            None => {
                                assert!(ls.insert(addr, HashMap::<String, u64>::new()).is_none());
                                ls.get_mut(&addr).unwrap()
                            }
                        };
                        let segments: Vec<&str> = string.split(' ').collect();
                        let amount: u64 = segments[0].parse().unwrap();
                        let animal: String = String::from(segments[1]);
                        let val = *state.get(&animal).unwrap_or(&0);
                        let global_val = *gs.state.get(&animal).unwrap_or(&0);
                        state.insert(animal.clone(), val + amount);
                        gs.state.insert(animal, global_val + amount);
                        gs.changed = true;

                        let response: String = serde_json::to_string(state).unwrap();
                        response
                    };
                    println!("{}", response);
                    bw.write_all(response.as_bytes()).await.unwrap();
                    bw.flush().await.unwrap();
                    socket.shutdown().await.unwrap();
                });
            }
        }));
    }
    let listeners_len = listeners.len();
    let file_writer = tokio::spawn(async move {
        if listeners_len == 0 {
            return;
        }
        loop {
            let json_string = {
                let gs = global_state_file_writer.lock().unwrap();
                if gs.changed {
                    serde_json::to_string(&gs.state).unwrap()
                } else {
                    String::new()
                }
            };
            if json_string.len() > 0 {
                {
                    let last_written = global_state_file_writer.lock().unwrap().last_written;
                    if last_written < SystemTime::now().sub(Duration::from_secs(5)) {
                        if let Ok(_) = tokio::fs::write("global_state.json", &json_string).await {
                            let mut gs = global_state_file_writer.lock().unwrap();
                            gs.changed = false;
                            gs.last_written = SystemTime::now();
                            println!("Wrote to global_state.json: {}", json_string);
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });
    for listener in listeners {
        listener.await??;
    }
    file_writer.await?;
    Ok(())
}
