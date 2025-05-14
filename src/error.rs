use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnixError {
    #[error("Failed to open mapped memory")]
    OpenFailed,
    #[error("Memory map failed")]
    MapFailed
}

#[derive(Error, Debug)]
pub enum WindowsError {

}