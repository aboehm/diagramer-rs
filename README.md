# Diagramer

A sequence diagram generator webservice in Rust🦀 based on [rocket🚀](https://rocket.rs/).

## Features

* Fast simple JSON-API
* SVG export
* [Mermaid](https://mermaid.js.org/) diagram code export
* Small frontend with live update
* Client implementation

## Usage

Start `diagramer` directly

```
cargo run
```

or build a release version via

```
cargo build --release
```

## API usage

### With client

The client library is implemented in the module `diagramer::client`.

```rust
use diagramer::client::Client;

let client = Client::new();
let session = client.new_session("http://localhost:8000").await;
println!("New session url {}", session.session_url()); 
session.add_link("a", "b", Some("Request")).await;
session.add_link("b", "c", Some("Forward")).await;
session.add_link("c", "a", Some("Response")).await;
```

The [network based stress test](examples/server-stress-test.rs) also uses the client implementation.

### Direct HTTP access

Create a new session

```sh
curl -XPOST 'http://127.0.0.1:8000/api/new-session'
```

With a session ID and a URI to the session

```json
{
  "id": "2888964795923373081",
  "uri": "/api/session/2888964795923373081"
}
```

Add a link from `a` to `b` with label `with a label`

```sh
curl -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/api/session/2888964795923373081/links' -d '{"from":"a", "to":"b", "label":"with a label"}'
```

Get a JSON representation of the session

```sh
curl 'http://127.0.0.1:8000/api/session/2888964795923373081'
```

```json
{
  "id": 2888964795923373000,
  "links": [
    {
      "timestamp": 1676679312120,
      "from": "a",
      "to": "b",
      "label": "with a label",
      "id": 2
    }
  ],
  "last_link": 2,
  "mermaid_url": "/api/session/2888964795923373081/mermaid",
  "svg_url": "/api/session/2888964795923373081/svg"
}
```
