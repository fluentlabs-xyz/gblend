use crate::error::Error;
use std::{fs, path::PathBuf};

pub fn create_dir_if_not_exists(path: &PathBuf, force: bool) -> Result<(), Error> {
    if path.exists() {
        if !force {
            return Err(Error::InitializationError(format!(
                "Directory {} already exists. Use --force to overwrite.",
                path.display()
            )));
        }
    } else {
        fs::create_dir_all(path).map_err(|e| {
            Error::InitializationError(format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })?;
    }
    Ok(())
}

pub fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let dst_path = dst.join(path.file_name().unwrap());

        if ty.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(path, dst_path)?;
        }
    }
    Ok(())
}
