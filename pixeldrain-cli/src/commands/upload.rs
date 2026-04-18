use crate::Config;
use anyhow::Context as _;
use pin_project_lite::pin_project;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

pin_project! {
    struct AsyncReadProgressWrapper<R> {
        #[pin]
        reader: R,
        progress_bar: indicatif::ProgressBar,
    }
}

impl AsyncReadProgressWrapper<tokio::fs::File> {
    async fn from_path(path: &Path) -> anyhow::Result<Self> {
        let file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let len = metadata.len();

        let progress_bar = indicatif::ProgressBar::new(len);
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

        Ok(Self {
            reader: file,
            progress_bar,
        })
    }
}

impl<R> AsyncRead for AsyncReadProgressWrapper<R>
where
    R: AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.as_mut().project();

        let start = buf.filled().len();
        let result = this.reader.poll_read(cx, buf);
        let end = buf.filled().len();
        let change = u64::try_from(end - start).unwrap();
        self.progress_bar.inc(change);

        result
    }
}

#[derive(Debug, clap::Parser)]
#[command(about = "Upload a file")]
pub struct Options {
    #[arg(help = "The path to the file to upload")]
    path: PathBuf,
}

pub async fn exec(client: &pixeldrain::Client, options: Options) -> anyhow::Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let token = config
        .context("Missing config. Set one up with `pixeldrain config edit`")?
        .token
        .context("Missing token. Set one up with `pixeldrain config edit`")?;

    client.set_token(&token);

    let file_name = options
        .path
        .file_name()
        .context("The given path has no file name")?
        .to_str()
        .context("The given path is not unicode")?
        .to_string();
    let reader = AsyncReadProgressWrapper::from_path(&options.path).await?;
    let file = pixeldrain::FileUpload::from_async_read(file_name, reader);
    let response = client.upload_file(file).await?;

    println!("Id: {}", response.id);

    Ok(())
}
