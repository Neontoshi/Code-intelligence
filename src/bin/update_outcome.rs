//! Update outcome tracking for dead code verdicts

use clap::{Parser, Subcommand};
use code_intelligence::analysis::OutcomeTracker;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Update dead code outcomes")]
struct Args {
    /// Project directory
    project_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Mark a verdict as removed
    Removed {
        /// The verdict ID
        id: String,
        /// Git commit hash (optional)
        #[arg(long)]
        commit: Option<String>,
    },
    /// Mark a verdict as false positive
    FalsePositive {
        /// The verdict ID
        id: String,
        /// Reason for false positive
        reason: String,
    },
    /// List all pending verdicts
    List,
    /// Show statistics
    Stats,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut tracker = OutcomeTracker::new(&args.project_dir);

    match args.command {
        Command::Removed { id, commit } => {
            tracker.mark_removed(&id, commit.as_deref())?;
            println!("✅ Marked {} as removed", id);
        }
        Command::FalsePositive { id, reason } => {
            tracker.mark_false_positive(&id, &reason)?;
            println!("✅ Marked {} as false positive: {}", id, reason);
        }
        Command::List => {
            let pending = tracker.get_pending();
            if pending.is_empty() {
                println!("✅ No pending verdicts");
            } else {
                println!("📋 Pending verdicts ({} total):", pending.len());
                for v in pending {
                    println!(
                        "  - {} ({}:{} - {:.1}%)",
                        v.function_name,
                        v.file.split('/').last().unwrap_or(&v.file),
                        v.line,
                        v.confidence * 100.0
                    );
                    println!("    ID: {}", v.id);
                }
            }
        }
        Command::Stats => {
            let stats = tracker.get_stats();
            println!("📊 Outcome Statistics:");
            println!("   Total flagged: {}", stats.total_flagged);
            println!(
                "   Removed: {} ({:.1}%)",
                stats.removed_count,
                stats.removal_rate * 100.0
            );
            println!("   Kept: {}", stats.kept_count);
            println!("   Pending: {}", stats.pending_count);
            println!("   False positives: {}", stats.false_positive_count);
        }
    }

    Ok(())
}
