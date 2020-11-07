use std::sync::mpsc::Receiver;
use getset::{CopyGetters, Getters};

pub enum FileDialogType {
    OpenRom,
    LoadState,
    SaveState,
}

pub enum FileDialogResult {
    None,
    OpenRom(String),
    LoadState(String),
    SaveState(String),
}

/// This module handles file dialogs in a separate thread.
/// Unforutnately, it's necessary due to a bug in the winit event loop.
/// See https://github.com/rust-windowing/winit/issues/1698
#[derive(CopyGetters, Getters)]
pub struct FileDialogHandler {
    #[getset(get_copy = "pub")]
    is_open: bool,
    #[getset(get = "pub")]
    last_result: FileDialogResult,
    chan_rx: Option<Receiver<FileDialogResult>>,
}

impl FileDialogHandler {
    const STATE_FILTER_PATT: &'static [&'static str] = &["*.p8s"];
    const STATE_FILTER_DESC: &'static str = "pich8 State (*.p8s)";

    pub fn new() -> Self {
        Self {
            is_open: false,
            last_result: FileDialogResult::None,
            chan_rx: None,
        }
    }

    pub fn open_dialog(&mut self, dialog_type: FileDialogType) {
        self.is_open = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.chan_rx = Some(rx);

        std::thread::spawn(move || {
            let mut result = FileDialogResult::None;
            match dialog_type {
                FileDialogType::OpenRom => {
                    if let Some(file_path) = tinyfiledialogs::open_file_dialog("Open ROM", "", None) {
                        result = FileDialogResult::OpenRom(file_path);
                    }
                },
                FileDialogType::LoadState => {
                    if let Some(file_path) = tinyfiledialogs::open_file_dialog("Load State", "", Some((FileDialogHandler::STATE_FILTER_PATT, FileDialogHandler::STATE_FILTER_DESC))) {
                        result = FileDialogResult::LoadState(file_path);
                    }
                },
                FileDialogType::SaveState => {
                    if let Some(file_path) = tinyfiledialogs::save_file_dialog_with_filter("Save State", "", FileDialogHandler::STATE_FILTER_PATT, FileDialogHandler::STATE_FILTER_DESC) {
                        result = FileDialogResult::SaveState(if file_path.contains(".") { file_path } else { format!("{}.p8s", file_path) });
                    }
                },
            }

            tx.send(result).expect("Communication failed");
        });
    }

    pub fn check_result(&mut self) -> bool {
        let mut result = false;
        if self.chan_rx.is_some() {
            if let Some(chan) = self.chan_rx.as_ref() {
                if let Ok(dialog_result) = chan.try_recv() {
                    self.last_result = dialog_result;
                    self.is_open = false;
                    result = true;
                }
            }
        }

        result
    }
}