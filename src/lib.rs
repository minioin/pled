use std::fmt::Debug;
use std::ops::Deref;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
pub use serde::*;
use serde::{Deserialize, Serialize};
use std::result;
pub use sled::*;

pub use error::Error;
pub use error::Result;
use std::str::FromStr;
use serde::export::Formatter;

pub mod error;
pub mod serialize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Id(String);

impl Deref for Id {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self(format!("{}", id))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<String> for Id {
    fn into(self) -> String {
        self.0
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Self(s)
    }
}


impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Self(s.into())
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

pub trait Store {
    fn add_all<T: Serialize + Document>(&self, data: &Vec<&T>) -> Result<Vec<Id>>;
    fn update_all<T: Serialize + Document>(&self, data: &Vec<&T>) -> Result<()>;
    fn remove_all<T: Document>(&self, data: &Vec<Id>) -> Result<()>;
    fn get_all<T: Document + DeserializeOwned>(&self, skip: usize, take: usize) -> Vec<T>;

    fn get<T: Document + DeserializeOwned>(&self, ids: Vec<Id>) -> Result<Vec<T>>;

    fn add<T: Serialize + Document>(&self, data: &T) -> Result<Vec<Id>> {
        self.add_all(&vec![data])
    }
    fn update<T: Serialize + Document>(&self, data: &T) -> Result<()> {
        self.update_all(&vec![data])
    }

    fn remove<T: Document>(&self, id: Id) -> Result<()> {
        self.remove_all::<T>(&vec![id])
    }
}

impl Store for SledStore {
    fn add_all<T: Serialize + Document>(&self, data: &Vec<&T>) -> Result<Vec<Id>> {
        let all = self
            .sled
            .open_tree(T::COLLECTION_NAME)?
            .transaction(|tree| {
                let mut ids = Vec::new();
                for item in data {
                    let id = match item.id() {
                        Some(id) => id,
                        None => self.sled.generate_id()?.into(),
                    };
                    let serialized: Vec<u8> = crate::serialize::serialize(&item)
                        .map_err(ConflictableTransactionError::Abort)?;
                    tree.insert(id.as_str(), serialized)?;
                    ids.push(id);
                }
                Ok(ids)
            })?;
        Ok(all)
    }

    fn update_all<T: Serialize + Document>(&self, data: &Vec<&T>) -> Result<()> {
        self.sled
            .open_tree(T::COLLECTION_NAME)?
            .transaction(|tree| {
                for item in data {
                    let id = item
                        .id()
                        .ok_or(sled::Error::Unsupported("No id provided".to_string()))
                        .map_err(ConflictableTransactionError::Storage)?;
                    let serialized: Vec<u8> = crate::serialize::serialize(&item)
                        .map_err(ConflictableTransactionError::Abort)?;
                    tree.insert(id.as_str(), serialized)?;
                }
                Ok(())
            })?;
        Ok(())
    }

    fn remove_all<T: Document>(&self, data: &Vec<Id>) -> Result<()> {
        self.sled
            .open_tree(T::COLLECTION_NAME)?
            .transaction(|tree| {
                for item in data {
                    tree.remove(item.as_str())?;
                }
                Ok(())
            })?;
        Ok(())
    }

    fn get_all<T: Document + DeserializeOwned>(&self, skip: usize, take: usize) -> Vec<T> {
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

    fn get<T: Document + DeserializeOwned>(&self, ids: Vec<Id>) -> Result<Vec<T>> {
        let mut items: Vec<T> = Vec::new();
        let tree = self.sled.open_tree(T::COLLECTION_NAME)?;
        for id in ids {
            let item = tree.get(id.as_str());
                let item = tree.get(id.as_str())?.ok_or(Error::NoneError)?;
            let item  = crate::serialize::deserialize(&item)?;
            items.push(item);
        }
        Ok(items)
    }
}
