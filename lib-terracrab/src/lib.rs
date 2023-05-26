use core::any::Any;
use std::error::Error;
use std::ffi::OsStr;
use serde_json::Value;
use std::fs::{self, read, read_to_string, write, remove_file, File};
use std::path::{self, Path};
use std::io::Result;
use std::path::PathBuf;

#[derive(Debug)]
struct TFStateStore {
    path: PathBuf,
}

impl TFStateStore {
    // Has a caller burden to check lock before get or put
    // Useful when client is correctly implemented
    // Consider reimplementing if guarantees must be stronger
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
    use std::env::temp_dir;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn it_can_write_a_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = TFStateStore::new(temp_file.path().parent().unwrap()).unwrap();
        let id = "xxx";
        let put_action = store.put(id, &Value::Null);
        assert_eq!(put_action.is_ok(), true);
        let mut found_file: PathBuf = temp_file.path().parent().unwrap().into();
        found_file.push(id);
        assert_eq!(found_file.exists(), true);
    }

    #[test]
    fn it_can_get_an_existing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = TFStateStore::new(temp_file.path().parent().unwrap()).unwrap();
        let id = "xxx";
        let put_action = store.put(id, &Value::Null);
        assert_eq!(put_action.is_ok(), true);
        let mut found_file: PathBuf = temp_file.path().parent().unwrap().into();
        found_file.push(id);
        assert_eq!(found_file.exists(), true);
        let output = store.get(id);
        assert_eq!(output, Some(Value::Null));

    }

    #[test]
    fn it_can_lock() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = TFStateStore::new(temp_file.path().parent().unwrap()).unwrap();
        let id = "xxx";
        let lock = store.lock(id, &Value::Null);
        let mut found_lock: PathBuf = temp_file.path().parent().unwrap().into();
        found_lock.push(id);
        found_lock.set_extension("lock");
        assert_eq!(found_lock.exists(), true);
        assert_eq!(read_to_string(found_lock).unwrap(), "null");
    }
    #[test]
    fn it_can_unlock() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = TFStateStore::new(temp_file.path().parent().unwrap()).unwrap();
        let id = "xxx";
        let lock = store.lock(id, &Value::Null);
        let mut found_lock: PathBuf = temp_file.path().parent().unwrap().into();
        found_lock.push(id);
        found_lock.set_extension("lock");
        assert_eq!(found_lock.clone().exists(), true);
        assert_eq!(read_to_string(found_lock.clone()).unwrap(), "null");
        store.unlock(id, &Value::Null).unwrap();
        assert_eq!(found_lock.exists(), false);
    }
}
