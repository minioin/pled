pub mod error;
pub mod serialize;

pub use error::Error;
pub use error::Result;

pub use serde::*;
pub use sled::*;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Id(u64);
impl Deref for Id {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ItemOrList<T> {
    Item(T),
    List(Vec<T>),
}

impl<T: Debug> std::fmt::Display for ItemOrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct SledStore {
    sled: sled::Db,
}

impl SledStore {
    pub fn with(sled: sled::Db) -> Self {
        Self { sled }
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        let sled = sled::open(path)?;
        Ok(Self::with(sled))
    }
}

pub trait Document {
    const COLLECTION_NAME: &'static str;

    fn name(&self) -> &str {
        Self::COLLECTION_NAME
    }

    fn id(&self) -> Option<Id> {
        None
    }
}

impl SledStore {
    pub fn add<T: Serialize + Document>(&self, data: &T) -> Result<()> {
        let id = match data.id() {
            Some(id) => id,
            None => self.sled.generate_id()?.into(),
        };
        self.sled
            .open_tree(data.name())?
            .insert(id.to_be_bytes(), crate::serialize::serialize(&data)?)?;
        Ok(())
    }

    pub fn update<T: Serialize + Document>(&self, data: &T) -> Result<()> {
        let id = data.id().ok_or(Error::NoneError)?;
        self.sled
            .open_tree(data.name())?
            .insert(id.to_be_bytes(), crate::serialize::serialize(&data)?)?;
        Ok(())
    }

    pub fn remove<T: Document>(&self, id: u64) -> Result<()> {
        self.sled
            .open_tree(T::COLLECTION_NAME)?
            .remove(id.to_be_bytes())?;
        Ok(())
    }

    pub fn get_all<T>(&self, skip: usize, take: usize) -> Vec<T>
    where
        T: DeserializeOwned + Document,
    {
        self.sled
            .open_tree(T::COLLECTION_NAME)
            .unwrap()
            .iter()
            .values()
            .skip(skip)
            .take(take)
            .map(|item| item.unwrap())
            .map(|item| crate::serialize::deserialize(&item).unwrap())
            .collect()
    }
}
