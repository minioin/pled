pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database error: `{0}`")]
    Sled(#[from] sled::Error),

    #[cfg(feature = "serde_cbor")]
    #[error("De/serialization error: `{0}`")]
    CBOR(#[from] serde_cbor::Error),

    #[error("De/serialization error: `{0}`")]
    #[cfg(feature = "bincode")]
    Bincode(#[from] bincode::Error),

    #[error("Given value can't be none")]
    NoneError,
}
