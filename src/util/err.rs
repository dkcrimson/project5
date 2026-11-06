use color_eyre::config::HookBuilder;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid utf8: {0}")]
    Utf8(String),
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err.to_string())
    }
}

pub fn setup_eyre() {
    let (panic_hook, error_hook) = HookBuilder::default().display_env_section(false).display_location_section(true).into_hooks();

    error_hook.install().expect("error hook should only be initialized once");
    panic_hook.install();
}
