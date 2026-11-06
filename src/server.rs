use std::collections::BinaryHeap;
use std::str::FromStr;
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;
use reqwest::{Response, header};
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::command::Command;
use crate::util::net::CLIENT;

const HEADER: &str = "\n == BEGIN TRANSMISSION == \n\n";
const FOOTER: &str = "\n  == END TRANSMISSION ==  \n\n";

macro_rules! respond {
    ($socket:expr, $client_address:expr, $message:expr) => {
        let message = format!("{}{}{}", HEADER, String::from_utf8_lossy($message), FOOTER);
        if let Err(e) = $socket.write_all(message.as_bytes()).await {
            error!("Failed to respond to {}: {}", $client_address, e);
            return
        }
    };
}

static CONTENT_DISPOSITION_FILENAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    debug!("Compiling content disposition filename regex");
    Regex::new(r#"(?i)filename\s*=\s*["']?([^"';]+)["']?"#).expect("invalid regex")
});

static DOWNLOADER: LazyLock<Mutex<Downloader>> = LazyLock::new(|| Mutex::new(Downloader::new()));

pub async fn listen() -> crate::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;

    let server_address = listener.local_addr()?;
    info!("Listening on {server_address}");

    loop {
        let (mut socket, client_address) = listener.accept().await?;
        info!("New connection from {client_address}");

        tokio::spawn(async move {
            let mut buf = [0; 4096];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        info!("Closed connection to {client_address}");
                        return
                    },
                    Ok(n) => {
                        let Ok(input) = str::from_utf8(&buf[0..n]) else {
                            warn!("Received invalid UTF-8 from {client_address}");
                            respond!(socket, client_address, b"Invalid UTF-8\n");
                            continue
                        };

                        trace!("Received '{}' from {client_address}", input.trim());

                        let Ok(command) = Command::from_str(input) else {
                            warn!("Received invalid command '{input}' from {client_address}");
                            respond!(socket, client_address, b"Invalid command\n");
                            continue
                        };

                        let response = evaluate_command(&command).await;
                        trace!("Responding to {client_address} with '{}'", response.trim());
                        respond!(socket, client_address, response.as_bytes());
                    },
                    Err(e) => {
                        error!("Failed to read from socket: {e}");
                        return
                    }
                }
            }
        });
    }
}

const HELP_TEXT: &str = "\
Commands:
    Download [URL] [DESTINATION]    // Download a URL to a file
    Headers [URL]                   // Dump the headers from a URL
    Move [SOURCE] [DESTINATION]     // Move a file
    Copy [SOURCE] [DESTINATION]     // Copy a file
    Remove [PATH]                   // Remove a file
    Read [PATH]                     // Read a file's contents
    List [PATH]                     // List files
    Help                            // Display this help menu
";

fn filename_from_url(url: &str) -> &str {
    url.rsplit_once('/').map_or_else(|| url, |parts| {
        let ending = parts.1;
        ending.split_once('?').map_or(ending, |parts| parts.0)
    })
}

pub async fn evaluate_command(command: &Command) -> String {
    let mut s = String::new();

    match command {
        Command::Help => HELP_TEXT.to_string(),
        Command::Read(path) => match fs::read_to_string(path).await {
            Ok(contents) => contents,
            Err(e) => format!("Failed to read {path}: {e}\n"),
        }
        Command::List => match fs::read_dir(".").await {
            Ok(mut readdir) => {
                let mut entries = vec![];
                while let Ok(Some(entry)) = readdir.next_entry().await {
                    entries.push(entry);
                }

                for file_name in entries.iter().map(|entry| entry.file_name().to_string_lossy().to_string()) {
                    let _ = writeln!(s, "{file_name}");
                }

                s
            }
            Err(e) => format!("Failed to read directory: {e}\n"),
        }
        Command::Remove(path) => match fs::remove_file(path).await {
            Ok(()) => format!("Removed {path}\n"),
            Err(e) => format!("Failed to remove {path}: {e}\n"),
        }
        Command::Move(source, destination) => match fs::rename(source, destination).await {
            Ok(()) => format!("Moved {source} to {destination}\n"),
            Err(e) => format!("Failed to move {source} to {destination}: {e}\n"),
        }
        Command::Copy(source, destination) => match fs::copy(source, destination).await {
            Ok(n) => format!("Copied {n} bytes from {source} to {destination}\n"),
            Err(e) => format!("Failed to move {source} to {destination}: {e}\n"),
        }
        Command::Headers(url) => {
            let response = match CLIENT.get(url).send().await {
                Ok(r) => r,
                Err(e) => return format!("No response from {url}: {e}\n")
            };

            for header in response.headers() {
                let _ = writeln!(s, "{} = {}", header.0.as_str(), String::from_utf8_lossy(header.1.as_bytes()));
            }

            s
        }
        Command::QueueDownload(url, destination) => {
            match DOWNLOADER.lock().await.enqueue(url, destination).await {
                Ok(()) => format!("Enqueued download of {url}\n"),
                Err(e) => format!("Failed to enqueue download of {url}: {e}\n"),
            }
        }
        Command::DownloadAll => {
            let queue = DOWNLOADER.lock().await.take_queue();
            Downloader::download_all(queue).await
        }
        Command::ShowDownloadQueue => {
            DOWNLOADER.lock().await.display_queue()
        }
    }
}

#[derive(Debug, Default)]
pub struct Downloader {
    queue: BinaryHeap<Download>,
    counter: u64,
}

#[derive(Debug)]
pub struct Download {
    content_length: Option<u64>,
    priority: u64,
    id: u64,
    destination: String,
    response: Response,
}

impl PartialEq for Download {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Download {}

impl PartialOrd for Download {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Download {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl Downloader {
    #[inline] #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_queue(&self) -> String {
        let mut s = String::new();

        for download in &self.queue {
            let _ = writeln!(s, "- {} ::: {}", download.content_length.map(|n| n.to_string()).unwrap_or_else(|| String::from("?")), download.response.url().as_str());
        }

        if s.is_empty() {
            let _ = writeln!(s, "Download queue is empty!");
        }

        s
    }

    pub async fn enqueue(&mut self, url: &str, destination: &Option<String>) -> crate::Result<()> {
        let response = CLIENT.get(url).send().await?;
        let final_url = response.url();

        let content_length = response.headers().get(header::CONTENT_LENGTH)
            .and_then(|header| header.to_str().ok())
            .and_then(|s| u64::from_str(s).ok())
            .inspect(|len| debug!("Parsed content length for {url}: {len}"));

        let mut destination = match destination.as_deref() {
            Some(s) => s,
            None if let Some(content_disposition) = response.headers().get(header::CONTENT_DISPOSITION).and_then(|header| header.to_str().ok()) 
                && let Some(content_disposition_filename) = CONTENT_DISPOSITION_FILENAME_REGEX.captures(content_disposition).and_then(|caps| caps.get(1).map(|m| m.as_str())) =>
            {
                debug!("Parsed content disposition filename for {url}: {content_disposition_filename}");
                content_disposition_filename
            },
            None => filename_from_url(final_url.as_str()),
        }.to_string();

        if destination.is_empty() {
            destination = String::from("fallback-filename");
        }

        let download = Download {
            content_length,
            priority: u64::MAX - content_length.unwrap_or(u64::MAX),
            id: self.counter + 1,
            response,
            destination,
        };

        self.counter += 1;
        self.queue.push(download);
        Ok(())
    }

    pub fn take_queue(&mut self) -> BinaryHeap<Download> { std::mem::take(&mut self.queue) }

    pub async fn download_all(mut queue: BinaryHeap<Download>) -> String {
        let mut s = String::new();

        while let Some(download) = queue.pop() {
            s.push_str(&download.download().await);
        }

        if s.is_empty() {
            s.push_str("Nothing to download!\n");
        } else {
            info!("Emptied download queue");
        }

        s
    }
}

impl Download {
    async fn download(self) -> String {
        let destination = &self.destination;

        let url = self.response.url().to_string();
        let bytes = match self.response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return format!("No body from {url}: {e}\n")
        };

        let mut oo = match OpenOptions::new().write(true).truncate(true).create(true).open(destination).await {
            Ok(oo) => oo,
            Err(e) => return format!("Cannot write to {}: {e}\n", self.destination),
        };

        if let Err(e) = oo.write_all(bytes.as_ref()).await {
            return format!("Failed to write to {}: {e}\n", destination)
        }

        info!("Downloaded {url} to {destination}");
        format!("Wrote {} bytes to {}\n", bytes.len(), destination)
    }
}
