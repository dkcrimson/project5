use std::sync::LazyLock;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build reqwest client")
});
