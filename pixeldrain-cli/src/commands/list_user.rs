use crate::Config;
use anyhow::Context;

#[derive(Debug, clap::Parser)]
#[command(about = "List the files from the current user")]
pub struct Options {}

pub async fn exec(client: &pixeldrain::Client, _options: Options) -> anyhow::Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let token = config
        .context("Missing config. Set one up with `pixeldrain config edit`")?
        .token
        .context("Missing token. Set one up with `pixeldrain config edit`")?;

    client.set_token(&token);

    let response = client.list_user_files().await?;

    for file in response.files {
        println!("{}", file.id);
        println!("  Name: {}", file.name);
        println!("  Views: {}", file.views);
        println!("  Downloads: {}", file.downloads);
        println!("  Sha256: {}", file.hash_sha256);
        println!();
    }

    Ok(())
}
