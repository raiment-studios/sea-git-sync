use anyhow::{Context, Result};
use clap::Parser;
use snowfall_core::prelude::cprintln;
use std::collections::HashSet;
use std::fs;
use std::fs::read_link;
use std::path::Path;
use std::process::Command;

/// CLI arguments for the sync tool
#[derive(Parser, Debug)]
#[command(name = "🌊 sea-git-sync")]
#[command(about = "A CLI tool to sync subdirectories from monorepos to external git repositories")]
struct Args {
    /// Remote repository URL
    #[arg(long, required = true)]
    remote: String,
    #[arg(long, default_value = "main")]
    branch: String,
    #[arg(long, default_value = "Sync changes")]
    message: String,
    /// Copy symlinks as files instead of links
    #[arg(long, default_value_t = true)]
    copy_symlinks: bool,
}

const SNAPSHOT_FILE: &str = ".git-sync-snapshot.tar.gz";

fn main() -> Result<()> {
    let start = std::time::Instant::now();

    let cargo_toml = include_str!("../Cargo.toml");
    let cargo_toml: toml::Value =
        toml::from_str(cargo_toml).context("Failed to parse Cargo.toml")?;
    let version = cargo_toml
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let args = Args::parse();
    cprintln!("#39C", "🌊 [sea-git-sync](#39C) [v{}](#B4F)", version);
    cprintln!("#39C", "{}", "[~](#39F)[~](#7AF)".repeat(32));
    if let Err(e) = sync_to_remote(&args) {
        eprintln!("Sync failed: {}", e);
        std::process::exit(1);
    }

    let duration = start.elapsed().as_secs_f32();
    println!();
    cprintln!(
        "#1C3",
        "✔ Sync completed successfully! [({duration:.1}s)](#666)",
    );
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

    let mut replaced_symlinks = Vec::new();
    if args.copy_symlinks {
        cprintln!("#39C", "Copying symlinks as files...");
        replaced_symlinks = copy_symlinks();
        for rep in &replaced_symlinks {
            git(&["add", "--force", rep.symlink_path.to_str().unwrap()])?;
        }
    }

    git(&["add", "."])?;
    git(&["commit", "-m", &args.message])?;
    git(&["pull", &args.remote, &args.branch, "--no-ff"])?;

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

    if !replaced_symlinks.is_empty() {
        cprintln!("#39C", "Restoring original symlinks...");
        undo_symlink_replacements(replaced_symlinks);
    }

    fs::remove_dir_all(git_dir).context("Failed to clean up .git directory")?;
    Ok(())
}

/// Struct to track replaced symlinks for undoing changes
#[derive(Debug)]
struct SymlinkReplacement {
    symlink_path: std::path::PathBuf,
    target: std::path::PathBuf,
    was_dir: bool,
}

/// Replace symlinks with their target directories, returning info for undoing changes
fn copy_symlinks() -> Vec<SymlinkReplacement> {
    fn visit_and_replace_symlinks(
        path: &Path,
        replaced: &mut Vec<SymlinkReplacement>,
        visited: &mut HashSet<std::path::PathBuf>,
    ) {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !visited.insert(entry_path.clone()) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.file_type().is_symlink() {
                let target = match read_link(&entry_path) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let abs_target = if target.is_absolute() {
                    target.clone()
                } else {
                    entry_path.parent().unwrap_or(Path::new(".")).join(&target)
                };

                let target_meta = match fs::metadata(&abs_target) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if target_meta.is_dir() {
                    let _ = fs::remove_file(&entry_path);
                    let _ = copy_dir_all(&abs_target, &entry_path);
                    let abs_target = abs_target.canonicalize().unwrap_or(abs_target);
                    replaced.push(SymlinkReplacement {
                        symlink_path: entry_path.clone(),
                        target: abs_target,
                        was_dir: true,
                    });
                    cprintln!("#555", "{}", entry_path.display());
                    visit_and_replace_symlinks(&entry_path, replaced, visited);
                }
                continue;
            }

            if metadata.is_dir() {
                visit_and_replace_symlinks(&entry_path, replaced, visited);
            }
        }
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    let mut replaced = Vec::new();
    let mut visited = HashSet::new();
    visit_and_replace_symlinks(Path::new("."), &mut replaced, &mut visited);
    replaced
}

/// Undo the symlink replacements, restoring the original symlinks
fn undo_symlink_replacements(replacements: Vec<SymlinkReplacement>) {
    for rep in replacements {
        let _ = fs::remove_dir_all(&rep.symlink_path);
        cprintln!("#555", "{}", rep.symlink_path.display());
        let _ = std::os::unix::fs::symlink(&rep.target, &rep.symlink_path);
    }
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
