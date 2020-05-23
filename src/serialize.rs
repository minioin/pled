use crate::error::Error;
use crate::error::Result;

#[cfg(feature = "serde_cbor")]
pub use serde_cbor::*;

#[cfg(feature = "bincode")]
pub use bincode::*;

#[cfg(feature = "bincode")]
pub fn serialize<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    bincode::serialize(value).map_err(Error::Bincode)
}

#[cfg(feature = "bincode")]
pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    bincode::deserialize(bytes).map_err(Error::Bincode)
}

#[cfg(feature = "serde_cbor")]
pub fn serialize<T: Sized>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    serde_cbor::to_vec(value).map_err(Error::CBOR)
}

#[cfg(feature = "serde_cbor")]
pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: serde::Deserialize<'a>,
{
    serde_cbor::from_slice(bytes).map_err(Error::CBOR)
}
