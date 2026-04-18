#[derive(Debug, serde::Deserialize)]
pub struct File {
    pub bandwidth_used: u64,
    pub downloads: u64,
    pub hash_sha256: String,
    pub id: String,
    pub mime_type: String,
    pub name: String,
    pub views: u64,
}

/// The user list response
#[derive(Debug, serde::Deserialize)]
pub struct ListUserFilesResponse {
    /// User files
    pub files: Vec<File>,
}
