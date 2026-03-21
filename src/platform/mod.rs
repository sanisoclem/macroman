use anyhow::Result;

pub trait InputDriver: Send + Sized + 'static {
    fn create() -> Result<Self>;
    fn press_key(&mut self, key: &str) -> Result<()>;
    fn release_key(&mut self, key: &str) -> Result<()>;

    fn wait_ms(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxDriver as PlatformDriver;

mod stub;
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
pub use stub::StubDriver as PlatformDriver;
