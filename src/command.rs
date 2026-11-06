use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    QueueDownload(String, Option<String>),
    DownloadAll,
    ShowDownloadQueue,
    Headers(String),
    Move(String, String),
    Copy(String, String),
    Remove(String),
    Read(String),
    List,
    Help,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["queue-download", url] => Ok(Self::QueueDownload(url.to_string(), None)),
            ["queue-download", url, destination] => Ok(Self::QueueDownload(url.to_string(), Some(destination.to_string()))),
            ["download-all"] => Ok(Self::DownloadAll),
            ["show-download-queue"] => Ok(Self::ShowDownloadQueue),
            ["headers", url] => Ok(Self::Headers(url.to_string())),
            ["move", source, destination] => Ok(Self::Move(source.to_string(), destination.to_string())),
            ["copy", source, destination] => Ok(Self::Copy(source.to_string(), destination.to_string())),
            ["remove", path] => Ok(Self::Remove(path.to_string())),
            ["read", path] => Ok(Self::Read(path.to_string())),
            ["list"] => Ok(Self::List),
            ["help"] | ["help", ..] => Ok(Self::Help),

            v => Err(format!("unsupported command: {v:?}"))
        }
    }
}
