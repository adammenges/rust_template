//! Only platform adapters may access AppKit. Linux has no tray or background service.
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{NativeApplication, StatusItem};

#[cfg(target_os = "linux")]
pub struct StatusItem;
#[cfg(target_os = "linux")]
impl StatusItem {
    pub fn install() -> anyhow::Result<Self> {
        Ok(Self)
    }
    pub fn take_actions(&self) -> u8 {
        0
    }
}
pub const OPEN: u8 = 1;
pub const QUIT: u8 = 2;

#[cfg(target_os = "linux")]
pub struct NativeApplication;
#[cfg(target_os = "linux")]
impl NativeApplication {
    pub fn install() -> anyhow::Result<Self> {
        Ok(Self)
    }
    pub fn request_termination(&self) {}
    pub fn take_quit_request(&self) -> bool {
        false
    }
    pub fn allow_termination(&self) {}
    pub fn reply_if_pending(&self) {}
}
