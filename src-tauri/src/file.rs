use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};

#[derive(Debug)]
pub enum FileError {
    PathResolveError,
    ReadDirError,
    FileTypeError,
    NotDirError,
    NotFileError
}

// 获取目录中的文件列表
pub fn get_dir_entries(path: Option<PathBuf>) -> Result<ReadDir, FileError> {
    if let None = path {
        return Err(FileError::PathResolveError);
    }
    // 读取文件夹列表
    let read_dir = fs::read_dir(path.unwrap());
    if let Err(_) = read_dir {
        return Err(FileError::ReadDirError);
    }
    Ok(read_dir.unwrap())
}
