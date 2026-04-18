use crate::Error;
use crate::FileInfo;
use crate::FileUpload;
use crate::ListUserFilesResponse;
use crate::UploadFileResponse;
use base64::prelude::*;
use reqwest::header::AUTHORIZATION;
use std::sync::Arc;

#[derive(Debug)]
struct ClientState {
    token: Option<String>,
}

/// The client
#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    state: Arc<std::sync::Mutex<ClientState>>,
}

impl Client {
    /// Make a new client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            state: Arc::new(std::sync::Mutex::new(ClientState { token: None })),
        }
    }

    /// Set the token.
    pub fn set_token(&self, token: &str) {
        let token = format!(":{token}");
        let mut encoded_token =
            String::with_capacity(base64::encoded_len(token.len(), true).unwrap_or(0));
        BASE64_STANDARD.encode_string(token, &mut encoded_token);

        self.state.lock().expect("state poisoned").token = Some(encoded_token);
    }

    /// Try to get the token.
    pub fn try_get_token(&self) -> Option<String> {
        self.state.lock().expect("state poisoned").token.clone()
    }

    /// Get the token.
    pub fn get_token(&self) -> Result<String, Error> {
        self.try_get_token().ok_or(Error::MissingToken)
    }

    /// List user files.
    ///
    /// This function requires a token.
    pub async fn list_user_files(&self) -> Result<ListUserFilesResponse, Error> {
        let token = self.get_token()?;
        let response = self
            .client
            .get("https://pixeldrain.com/api/user/files")
            .header(AUTHORIZATION, format!("Basic {token}"))
            .send()
            .await?
            .error_for_status()?;

        let value: ListUserFilesResponse = response.json().await?;

        Ok(value)
    }

    /// Upload a file.
    ///
    /// This function requires a token.
    pub async fn upload_file(&self, file: FileUpload) -> Result<UploadFileResponse, Error> {
        let token = self.get_token()?;

        let response = self
            .client
            .put(format!(
                "https://pixeldrain.com/api/file/{}",
                file.file_name
            ))
            .header(AUTHORIZATION, format!("Basic {token}"))
            .body(file.body)
            .send()
            .await?
            .error_for_status()?;

        let value: UploadFileResponse = response.json().await?;

        Ok(value)
    }

    /// Get info about a file.
    ///
    /// This function does NOT require a token.
    pub async fn get_file_info(&self, id: &str) -> Result<FileInfo, Error> {
        let token = self.try_get_token();

        let mut request = self
            .client
            .get(format!("https://pixeldrain.com/api/file/{id}/info"));
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Basic {token}"));
        }
        let response = request.send().await?.error_for_status()?;
        let value: FileInfo = response.json().await?;

        Ok(value)
    }

    /// Download a file.
    ///
    /// This function does NOT require a token.
    pub async fn download_file(&self, id: &str) -> Result<reqwest::Response, Error> {
        let token = self.try_get_token();

        let mut request = self
            .client
            .get(format!("https://pixeldrain.com/api/file/{id}"));
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Basic {token}"));
        }
        let response = request.send().await?.error_for_status()?;

        Ok(response)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
