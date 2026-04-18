mod client;
mod model;

pub use self::client::Client;
pub use self::model::ListUserFilesResponse;
pub use self::model::UploadFileResponse;
pub use reqwest::Body;
use std::path::Path;
use tokio_util::io::ReaderStream;

/// A file upload
#[derive(Debug)]
pub struct FileUpload {
    /// The file name
    pub file_name: String,

    /// The file body
    pub body: Body,
}

impl FileUpload {
    pub async fn from_path<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| std::io::Error::other("Missing file name"))?
            .to_string();

        let file = tokio::fs::File::open(path).await?;
        let body = reqwest::Body::from(file);

        Ok(Self { file_name, body })
    }

    pub fn from_async_read<R>(file_name: String, reader: R) -> Self
    where
        R: tokio::io::AsyncRead + Send + 'static,
    {
        Self {
            file_name,
            body: reqwest::Body::wrap_stream(ReaderStream::new(reader)),
        }
    }
}

/// Library error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),

    #[error("missing token")]
    MissingToken,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::LazyLock;

    const TOKEN: LazyLock<String> =
        LazyLock::new(|| std::fs::read_to_string("token.txt").expect("failed to load token.txt"));

    #[tokio::test]
    async fn user_list_works() {
        let client = Client::new();
        client.set_token(&TOKEN);

        let response = client.list_user_files().await.expect("failed to list");
        dbg!(response);
    }
}
