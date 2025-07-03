use std::fs;
use std::path::Path;

use crate::core::error::fs_errors::FsError;

pub fn read_file(path: &str) -> Result<String, FsError> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(FsError::FileNotFound(format!("{}", path)));
    }
    if !path_obj.is_file() {
        return Err(FsError::NotAFile(format!("{}", path)));
    }

    match fs::read_to_string(path_obj) {
        Ok(content) => Ok(content),
        Err(_) => Err(FsError::ReadError(format!("{}", path))),
    }
}
