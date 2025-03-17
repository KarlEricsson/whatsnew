use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use etcetera::BaseStrategy;
use whatsnew_core::UserData;

pub mod commits;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct App {
    #[command(flatten)]
    pub global_opts: GlobalOpts,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Commits {
        #[command(subcommand)]
        command: CommitsCommands,
    },
}

#[derive(Subcommand)]
pub enum CommitsCommands {
    Add { input: String },
    Remove { input: String },
    List,
    Check,
}
pub fn get_default_data_file() -> PathBuf {
    let data_dir = etcetera::choose_base_strategy()
        .expect("Failed to get data directory")
        .data_dir();
    data_dir.join("whatsnew/data.json")
}

#[derive(Args)]
pub struct GlobalOpts {
    #[arg(global = true, long, default_value_os_t = get_default_data_file())]
    pub data_file: PathBuf,

    #[arg(global = true, long)]
    pub skip_update: bool,
    #[arg(long, value_enum, default_value_t = Color::Auto)]
    pub color: Color,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl Color {
    pub fn init(self) {
        match self {
            Self::Auto => {}
            Self::Always => owo_colors::set_override(true),
            Self::Never => owo_colors::set_override(false),
        }
    }
}

impl App {
    pub async fn handle_command(self, mut userdata: UserData) -> Result<()> {
        match self.command {
            Some(Commands::Commits { command }) => match command {
                CommitsCommands::Add { input } => commits::add(&mut userdata, &input)?,
                CommitsCommands::Remove { input } => commits::remove(&mut userdata, &input)?,
                CommitsCommands::List => commits::list(&userdata)?,
                CommitsCommands::Check => commits::check(&mut userdata, &self.global_opts).await?,
            },
            None => commits::check(&mut userdata, &self.global_opts).await?,
        }
        userdata.save_to_file(&self.global_opts.data_file)?;
        Ok(())
    }
}
