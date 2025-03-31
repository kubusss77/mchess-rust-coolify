use std::io;

use mchess::server;

#[tokio::main]
async fn main() -> io::Result<()> {
    if let Err(e) = server::run_server().await {
        eprintln!("Server error: {}", e);
        return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
    }

    Ok(())
}