use std::fs;

use anyhow::{Ok, Result};
use clap::Parser;
use whatsnew_core::UserData;

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}

async fn run() -> Result<()> {
    let app = whatsnew_cli::commands::App::parse();
    app.global_opts.color.clone().init();
    let data_file = &app.global_opts.data_file;
    if !data_file.exists() {
        if let Some(parent) = data_file.parent() {
            fs::create_dir_all(parent)?;
        }
        UserData::new().save_to_file(data_file)?;
    }

    let userdata = UserData::load_from_file(&app.global_opts.data_file)?;

    app.handle_command(userdata).await?;

    Ok(())
}
