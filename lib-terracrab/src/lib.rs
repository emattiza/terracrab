use core::any::Any;
use std::error::Error;
use std::ffi::OsStr;
use serde_json::Value;
use std::fs::{self, read, read_to_string, write, remove_file};
use std::path::{self, Path};
use std::io::Result;
use std::path::PathBuf;

#[derive(Debug)]
struct TFStateStore {
    path: PathBuf,
}

impl TFStateStore {
    pub fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        Ok(TFStateStore {
            path: PathBuf::from(path),
        })
    }

    pub fn get(&self, id: &str) -> Option<Value> {
        let mut file_path = self.path.clone();
        file_path.push(id);
        if file_path.exists() {
            if file_path.is_file() {
                let data = read_to_string(file_path).ok()?;
                return serde_json::from_str(&data).ok()
            }
        }
        None
    }

    pub fn put(&self, id: &str, info: &Value) -> Result<()> {
        let mut path = self.path.clone();
        path.push(id);
        let data = serde_json::to_string(info)?;
        write(path, data)
    }

    pub fn lock(&self, id: &str, info: &Value) -> Result<(bool, Value)> {
        let mut path = self.path.clone();
        path.push(id);
        path.set_extension("lock");
        if path.exists() {
            let contents = read_to_string(path)?;
            let data = serde_json::from_str(&contents)?;
            return Ok((false, data))
        }
        let new_data = serde_json::to_string(info)?;
        write(path, new_data)?;
        Ok((true, Value::Null))
    }

    pub fn unlock(&self, id: &str, _info: &Value) -> Result<bool> {
        let mut path = self.path.clone();
        path.push(id);
        path.set_extension("lock");
        if path.exists() {
            remove_file(path)?;
            return Ok(true)
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
