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

    #[error("DatabaseError")]
    DatabaseError,
}

impl From<sled::TransactionError<Error>> for Error {
    fn from(t: sled::TransactionError<Error>) -> Self {
        match t {
            sled::TransactionError::Abort(t) => t,
            sled::TransactionError::Storage(t) => Error::Sled(t),
        }
    }
}
