mod util;
mod command;
mod server;

pub use util::err::{Error, Result};

#[macro_use]
extern crate tracing;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    util::log::log();
    util::err::setup_eyre();

    debug!("Hi!!!");
    server::listen().await?;

    Ok(())
}
