// muguet - muguet-server.rs
// Constantin Clerc (c22dev) - 0.1

#[path = "../proto.rs"]
mod proto;
#[path = "../server.rs"]
mod server;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(proto::DEFAULT_PORT);
    let max: usize = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(32);
    server::run(port, max)
}
