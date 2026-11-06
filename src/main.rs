mod util;

pub use util::err::{Error, Result};

#[macro_use]
extern crate tracing;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    util::log::log();
    util::err::setup_eyre();

    debug!("Hi!!!");

    Ok(())
}
