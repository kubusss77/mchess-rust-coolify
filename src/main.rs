use std::io;

use mchess::protocol::UciProtocol;

fn main() -> io::Result<()> {
    let mut uci = UciProtocol::new();
    uci.run()?;

    Ok(())
}