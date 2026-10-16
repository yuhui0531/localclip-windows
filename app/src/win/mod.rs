#[cfg(windows)]
pub mod clipboard;
#[cfg(windows)]
pub mod foreground;
pub mod formats;
#[cfg(windows)]
pub mod listener;
#[cfg(windows)]
pub mod ocr;
#[cfg(windows)]
pub mod paste;
#[cfg(windows)]
pub mod source;

#[cfg(all(windows, feature = "desktop"))]
pub mod dialog;
#[cfg(windows)]
pub mod rich;

#[cfg(windows)]
pub mod drag;

#[cfg(windows)]
pub mod code_file;

#[cfg(windows)]
pub mod preview;
