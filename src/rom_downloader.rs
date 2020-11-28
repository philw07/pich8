use std::sync::mpsc::Receiver;
use url::Url;

pub enum DownloadResult {
    Success(Vec<u8>),
    Fail(String),
    None,
}

pub struct RomDownloader {
    is_active: bool,
    chan_rx: Option<Receiver<DownloadResult>>,
}

impl RomDownloader {
    pub fn new() -> Self {
        Self {
            is_active: false,
            chan_rx: None,
        }
    }

    pub fn is_active(&self) -> bool { self.is_active }

    pub fn download(&mut self, url: Url) {
        self.is_active = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.chan_rx = Some(rx);
        
        std::thread::spawn(move || {
            let result = match reqwest::blocking::get(url) {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.bytes() {
                            Ok(data) => DownloadResult::Success(data.to_vec()),
                            Err(e) => DownloadResult::Fail(format!("Download failed: {}", e)),
                        }
                    } else {
                        DownloadResult::Fail("Download failed!".to_string())
                    }
                },
                Err(e) => DownloadResult::Fail(format!("Download failed: {}", e)),
            };

            tx.send(result).expect("Communication failed");
        });
    }

    pub fn check_result(&mut self) -> DownloadResult {
        let mut result = DownloadResult::None;
        if self.chan_rx.is_some() {
            if let Some(chan) = self.chan_rx.as_ref() {
                if let Ok(download_result) = chan.try_recv() {
                    self.is_active = false;
                    result = download_result;
                }
            }
        }

        result
    }
}