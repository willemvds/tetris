use std::collections;
use std::fmt;

use sdl2::rwops;

#[derive(Debug)]
pub struct ErrNotFound {}

impl fmt::Display for ErrNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset not found in registry")
    }
}

pub struct Registry<'r> {
    assets: collections::HashMap<&'r str, Vec<u8>>,
}

impl<'r> Registry<'r> {
    pub fn new() -> Registry<'r> {
        Registry {
            assets: collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: &'r str, bytes: Vec<u8>) {
        self.assets.insert(path, bytes);
    }

    pub fn get(&self, path: &str) -> Result<&Vec<u8>, ErrNotFound> {
        match self.assets.get(path) {
            Some(asset) => Ok(asset),
            None => Err(ErrNotFound {}),
        }
    }

    pub fn get_rwops(&self, path: &str) -> Result<rwops::RWops, ErrNotFound> {
        match self.assets.get(path) {
            Some(asset) => {
                if let Ok(rw) = rwops::RWops::from_bytes(asset) {
                    return Ok(rw);
                }
            }
            None => (),
        }

        Err(ErrNotFound {})
    }
}
