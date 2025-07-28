pub enum FsError {
    FileNotFound(String),
    NotAFile(String),
    ReadError(String),
    WriteError(String),
}
