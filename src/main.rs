mod cprintln;

use anyhow::{Context, Result};
use clap::Parser;
use cprintln::*;
use std::fs;
use std::iter::repeat;
use std::path::Path;
use std::process::Command;

/// CLI arguments for the sync tool
#[derive(Parser, Debug)]
#[command(name = "🌊 sea-git-publish")]
#[command(about = "A CLI tool to sync subdirectories from monorepos to external git repositories")]
struct Args {
    /// Remote repository URL
    #[arg(long, required = true)]
    remote: String,
    #[arg(long, default_value = "main")]
    branch: String,
    #[arg(long, default_value = "Sync changes")]
    message: String,
}

const SNAPSHOT_FILE: &str = ".git-sync-snapshot.tar.gz";

fn main() -> Result<()> {
    let args = Args::parse();
    cprintln!("#39C", "🌊 [sea-git-publish](#39C) [v0.1.2](#829)");
    cprintln!("#39C", "{}", "[~](#CCF)[~](#CCC)".repeat(32));
    if let Err(e) = sync_to_remote(&args) {
        eprintln!("Sync failed: {}", e);
        std::process::exit(1);
    }
    cprintln!("#1C3", "✔ Sync completed successfully!");
    Ok(())
}

fn sync_to_remote(args: &Args) -> Result<()> {
    let snapshot_path = Path::new(SNAPSHOT_FILE);
    if !snapshot_path.exists() {
        cprintln!("#39C", "No snapshot found, creating initial clone...");
        create_initial_snapshot(&args.remote)?;
    }

    cprintln!("#39C", "Syncing changes to remote repository...");

    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        ensure_clean_dir(git_dir)?;
        extract_snapshot(snapshot_path, git_dir)?;
    }
    // Remove the snapshot since we have an active .git directory
    run_command("rm", &["-f", ".git-sync-snapshot.tar.gz"])?;
    git(&["ls-files"])?;

    git(&["add", "."])?;
    git(&["commit", "-m", &args.message])?;
    git(&["pull", &args.remote, &args.branch])?;

    match git(&["push", &args.remote, &args.branch]) {
        Ok(_) => {
            cprintln!("#39C", "Push successful, updating snapshot...");
            git(&["gc", "--aggressive", "--prune=now"])?;
            create_snapshot(git_dir, snapshot_path)?;
        }
        Err(_) => eprintln!("Push failed, not updating snapshot"),
    }

    // Display the snapshot file size (since it can be abnormally large)
    run_command("du", &["-h", ".git-sync-snapshot.tar.gz"])?;

    fs::remove_dir_all(git_dir).context("Failed to clean up .git directory")?;
    Ok(())
}

/// Create initial snapshot by cloning the remote repository
fn create_initial_snapshot(remote_url: &str) -> Result<()> {
    let temp_dir = Path::new("git-remote");
    ensure_clean_dir(temp_dir)?;

    run_command_in_dir("git", &["clone", remote_url, "."], temp_dir)?;
    create_snapshot(&temp_dir.join(".git"), Path::new(SNAPSHOT_FILE))?;
    fs::remove_dir_all(temp_dir)?;
    Ok(())
}

/// Extract git snapshot to target directory
fn extract_snapshot(snapshot_path: &Path, target_dir: &Path) -> Result<()> {
    run_command(
        "tar",
        &[
            "-xzf",
            path_str(snapshot_path)?,
            "-C",
            path_str(target_dir)?,
            "--strip-components=1",
        ],
    )
}

/// Create compressed snapshot of git directory
fn create_snapshot(git_dir: &Path, snapshot_path: &Path) -> Result<()> {
    let parent = git_dir.parent().context("git directory has no parent")?;
    let name = git_dir
        .file_name()
        .context("Git directory has no name")?
        .to_str()
        .context("Invalid git directory name")?;

    let mut args = vec!["-czf", path_str(snapshot_path)?];
    if !parent.display().to_string().is_empty() {
        args.extend(vec!["-C", path_str(parent)?]);
    }
    args.push(name);
    run_command("tar", &args)
}

// Helper functions

/// Run a git command with standard error handling
fn git(args: &[&str]) -> Result<()> {
    cprintln!("555", "> [git {}](goldenrod)", args.join(" "));
    let status = Command::new("git")
        .args(args)
        .status()
        .context("Failed to execute git command")?;
    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);

        // For git commit, exit code 1 with no staged changes is acceptable
        if args[0] == "commit" && exit_code == 1 {
            println!("No changes to commit");
            return Ok(());
        }

        return Err(anyhow::anyhow!(
            "Git command failed with exit code: {}",
            exit_code
        ));
    }
    Ok(())
}

/// Run any command with error handling
fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    run_command_in_dir(cmd, args, Path::new("."))
}

/// Run command in specific directory
fn run_command_in_dir(cmd: &str, args: &[&str], dir: &Path) -> Result<()> {
    cprintln!("555", "> [{} {}](goldenrod)", cmd, args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to execute {} command", cmd))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Ensure directory exists and is empty
fn ensure_clean_dir(dir: &Path) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)
            .with_context(|| format!("Failed to remove existing directory: {}", dir.display()))?;
    }
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    Ok(())
}

/// Convert Path to &str with proper error handling
fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("Invalid path: {}", path.display()))
}
