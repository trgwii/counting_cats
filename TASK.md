# Task

The application layer of the client/server can be anything. Let's say you want to send `5 cats` which is `[53, 32, 99, 97, 116, 115]` in bytes. You could send it as is or using http or a custom one like providing a size for the request before the request. The size of `5 cats` is 6 bytes which means you could send `[6, 53, 32, 99, 97, 116, 115]` which has `6` to specify the size of the content.

## Client

Create a client. The client should start from a provided config retrieved from the stdin.

Example of how to provide the input:

- `cat endpoints.json | ./client`
- `echo '[{"socket_address": "127.0.0.1:3000","request": "5 cats"}]' | ./client`
- `./client` then paste the json content (then add a newline and press ctrl+d)

Example input:

```json
[
  {
    "socket_address": "127.0.0.1:3000",
    "request": "5 cats"
  },
  {
    "socket_address": "127.0.0.1:3001",
    "request": "16 chickens"
  },
  {
    "socket_address": "127.0.0.1:3001",
    "request": "9 dogs"
  },
  {
    "socket_address": "127.0.0.1:3002",
    "request": [51, 32, 98, 117, 110, 110, 105, 101, 115]
  }
]
```

With the example input above the client should create 4 tcp connections and send the request concurrently.

- The first connection `127.0.0.1:3000` should send `5 cats`.
- The second connection `127.0.0.1:3001` should send `16 chickens`.
- The third connection `127.0.0.1:3001` should send `9 dogs`.
- The fourth connection `127.0.0.1:3002` should send `3 bunnies` (`[51, 32, 98, 117, 110, 110, 105, 101, 115]`).

Print the values returned from the server.

## Server

Create a TCP server.

The server should start multiple TCP listeners based on the ports provided in the arguments. For example, `./server 3000 3001 3002` should start 3 TCP listeners on `127.0.0.1:3000`, `127.0.0.1:3001`, and `127.0.0.1:3002`.

The server should manage a state per listener and a global state with the combined state of the state per listener.

The states should keep track of how many animals are sent.

Example: If the client sends `2 birds` to `127.0.0.1:3000` and `3 birds` to `127.0.0.1:3001`. The global state should have `5 birds` and port 3000 should have `2 birds` and port 3001 should have `3 birds`.

The server should respond with the listener state to the client.

Example:

1. If the client sends `2 birds` to `127.0.0.1:3000`.
   - The listener state should be `{"birds": 2}`
   - The global state should be `{"birds": 2}`
   - The server should respond with the listener state `{"birds": 2}`
2. If the client sends `3 birds` to `127.0.0.1:3000`.
   - The listener state should be `{"birds": 5}`
   - The global state should be `{"birds": 5}`
   - The server should respond with the listener state `{"birds": 5}`
3. If the client sends `4 birds` to `127.0.0.1:3001`.
   - The listener state should be `{"birds": 4}`
   - The global state should be `{"birds": 9}`
   - The server should respond with the listener state `{"birds": 4}`
4. If the client sends `9 cats` to `127.0.0.1:3000`.
   - The listener state should be `{"birds": 5, "cats": 9}`
   - The global state should be `{"birds": 9, "cats": 9}`
   - The server should respond with the listener state `{"birds": 5, "cats": 9}`

The global state should be written to the file. Since writing to disk would be very expensive if it's done every time the state updates, try to only write to disk every 5s. Don't write to disk if nothing has changed.

Example state output file:

```json
{
  "bunnies": 30,
  "cats": 50,
  "dogs": 90,
  "chickens": 160
}
```
