# Here is the high level plan

1. Follow [tsoding tutorial](https://www.youtube.com/watch?v=BbIEuNscn_E) and implement simple multi-user chat using only standard library ⏳
  - server, client, auth, rate limiting ✅
  - tls and security - 📌
2. Add separate client with UI using crossterm ✅
3. Add TUI using ratatui (but first just with crossterm) ✅  
4. Break it trying to optimize something 📌
5. Add some protocol to handle 2 problems: "if message is rate limited the client doesn't know that", "message framing" 📌
6. Rewrite transport with async (tokio?)
7. ... "no tengo ni una puta idea" what i will do next


## Additional things:
- add the ability to start client without ip-address
- add commands to connect, disconnect and help
- add status bar (to show connected/disconnected for now) ✅
  


## To fix
  - unwraps in senders
  - need to make widget for messages scrollable to show only last N messages if count is more than height of the area! 


## Start instructions

### Server
```console
$ cargo run --bin server
Token: <generated token>
<logs>
```
### Client
```console
$ cargo run --bin client 127.0.0.1
<paste or type token from server>
<type messages>
```
