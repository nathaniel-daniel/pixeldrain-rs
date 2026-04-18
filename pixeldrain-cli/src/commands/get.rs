use crate::Config;
use anyhow::Context;
use anyhow::ensure;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, clap::Parser)]
#[command(about = "Download a file")]
pub struct Options {
    #[arg(help = "The url to download from")]
    url: Url,

    #[arg(help = "The path to the new file")]
    output: Option<PathBuf>,
}

fn extract_id(url: &Url) -> anyhow::Result<String> {
    ensure!(url.host_str() == Some("pixeldrain.com"));

    let mut path_iter = url.path_segments().context("Missing path")?;
    ensure!(path_iter.next() == Some("u"));
    let id = path_iter.next().context("Missing id")?;
    ensure!(path_iter.next().is_none());

    Ok(id.to_string())
}

pub async fn exec(client: &pixeldrain::Client, options: Options) -> anyhow::Result<()> {
    let config = Config::load().context("Failed to load config")?;
    if let Some(token) = config.and_then(|config| config.token) {
        client.set_token(&token);
    }

    let file_id = extract_id(&options.url)?;

    let file_info = client
        .get_file_info(&file_id)
        .await
        .context("Failed to get file info")?;
    let output = match options.output {
        Some(output) => output,
        None => file_info.name.into(),
    };
    ensure!(
        !tokio::fs::try_exists(&output).await?,
        "File already exists"
    );

    let temp_path = output.with_added_extension("part");
    let mut file = tokio::fs::File::create(&temp_path).await?;

    let mut file_response = client.download_file(&file_id).await?;

    let content_length = file_info.size;

    let progress_bar = indicatif::ProgressBar::new(content_length);
    let progress_bar_style_template = "[Time = {elapsed_precise} | ETA = {eta_precise} | Speed = {bytes_per_sec}] {wide_bar} {bytes}/{total_bytes}";
    let progress_bar_style = indicatif::ProgressStyle::default_bar()
        .template(progress_bar_style_template)
        .expect("invalid progress bar style template");
    progress_bar.set_style(progress_bar_style);

    {
        let progress_bar = progress_bar.clone();
        tokio::spawn(async move {
            while !progress_bar.is_finished() {
                progress_bar.tick();
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });
    }

    let mut written = 0;
    while let Some(chunk) = file_response.chunk().await? {
        let chund_len_u64 = u64::try_from(chunk.len())?;
        file.write_all(&chunk).await?;
        written += chund_len_u64;
        progress_bar.inc(chund_len_u64);
    }
    ensure!(written == content_length);

    file.flush().await?;
    file.sync_all().await?;
    tokio::fs::rename(temp_path, &output).await?;

    Ok(())
}
