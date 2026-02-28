# luanti-rs

Luanti Libraries and Tools written in Rust.

## Starting a Demo Server

There's a basic demo server implementation covering basic loading of an existing world and mapgen.

To start a server run the following command:

```sh
cargo run --package demo-server -- --listen 40000
```

The server will accept connections from a regular Luanti client (5.11.0) on port 40000 with any
user name and an empty password. Other Clients versions _may_ work as well.
