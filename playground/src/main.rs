//! This is just a playground to play with it while developing.
//!
//! When the library is finished, this file shall be removed.

use dbus_stream::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    smol::block_on(async {
        let system = Connection::new_system().await;

        system
    })?;

    Ok(())
}
