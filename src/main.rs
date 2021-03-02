/// This is just a playground to play with it while developing.
///
/// When the library is finished, this file shall be removed.

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    smol::block_on(async {
        let system = dbus::connection::Connection::new_system().await;

        system
    })?;

    Ok(())
}
