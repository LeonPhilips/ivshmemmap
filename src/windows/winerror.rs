use anyhow::{bail, Result};
use std::fmt::Formatter;
use windows::imp::GetLastError;

#[derive(Copy, Clone)]
pub struct WindowsError(u32);

impl WindowsError {
    #[must_use]
    pub fn current() -> Self {
        Self(unsafe { GetLastError() })
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        self.0 != 0
    }

    pub fn check(&self) -> Result<()> {
        if self.is_error() {
            bail!("{:?}", self)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Debug for WindowsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error code: {:?} {:#02x}", self.0, self.0)
    }
}
