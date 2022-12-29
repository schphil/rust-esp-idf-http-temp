use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Anyhow Error")]
    AnyhowError(#[from] anyhow::Error),
    #[error("Esp Error")]
    EspError(#[from] esp_idf_sys::EspError),
    #[error("Std Error")]
    StdError(#[from] std::io::Error),
}
