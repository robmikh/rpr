use windows::{
    core::{Result, HRESULT},
    Win32::Foundation::WIN32_ERROR,
};

pub trait ToWindowsResult {
    fn ok(self) -> Result<()>;
}

impl ToWindowsResult for WIN32_ERROR {
    fn ok(self) -> Result<()> {
        let hresult: HRESULT = self.into();
        hresult.ok()
    }
}
