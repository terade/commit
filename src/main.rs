#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::str_to_string
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::struct_excessive_bools
)]

mod commit;
mod commit_message;
mod config;

use anyhow::Result;
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;

use commit::{
    check_staged_files, commit, git_add_all_modified, pre_commit_check, read_cached_commit,
    write_cached_commit,
};
use commit_message::make_message_commit;

const DEFAULT_CONFIG_FILE: &str = include_str!("../commit-default.json");

#[derive(Parser, Debug)]
#[command(about, author, version)]
struct Args {
    /// Custom configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Init custom configuration file
    #[arg(long)]
    init: bool,
    /// Use as hook
    #[arg(long)]
    hook: bool,
    /// Retry commit with the same message as the last one
    #[arg(short, long)]
    retry: bool,
    /// Add all modified files into staging
    #[arg(short, long)]
    all: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.all {
        git_add_all_modified()?;
    }

    check_staged_files()?;

    if args.init {
        let mut file = std::fs::File::create("commit.json")?;
        file.write_all(DEFAULT_CONFIG_FILE.as_bytes())?;
        return Ok(());
    }

    let pattern = config::get_pattern(args.config)?;

    if args.retry {
        let commit_message = read_cached_commit()?;
        pre_commit_check(pattern.config.pre_commit, &commit_message)?;
        commit(&commit_message)?;
        return Ok(());
    }

    let commit_message = make_message_commit(pattern.clone())?;
    write_cached_commit(&commit_message)?;

    pre_commit_check(pattern.config.pre_commit, &commit_message)?;

    if args.hook {
        return Ok(());
    }

    commit(&commit_message)?;
    Ok(())
}
