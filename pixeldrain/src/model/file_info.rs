/// Info about a file
#[derive(Debug, serde::Deserialize)]
pub struct FileInfo {
    /// The id of the file.
    pub id: String,

    /// The name of the file.
    pub name: String,

    /// The size of the file.
    pub size: u64,

    /// The number of views
    pub views: u64,

    /// The number of downloads.
    pub downloads: u64,

    /// The sha256 hash of the file.
    pub hash_sha256: String,
}
