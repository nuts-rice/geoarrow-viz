use thiserror::Error;
#[derive(Debug, Error)]
pub enum GeoArrowError {
    #[error("i/o error: {0}")]
    Io(String),
    #[error("Arrow error: {0}")]
    Arrow(String),
    #[error("Parquet Error: {0}")]
    Parquet(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("WebAssembly error: {0}")]
    Wasm(String),

}
