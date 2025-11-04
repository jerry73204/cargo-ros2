mod cache;
mod config_patcher;
mod dependency_parser;
mod workflow;

use clap::{Parser, Subcommand};
use eyre::Result;
use std::env;
use std::path::PathBuf;
use workflow::WorkflowContext;

/// All-in-one build tool for ROS 2 Rust projects
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Ros2(Ros2Args),
}

#[derive(Debug, Parser)]
#[command(name = "ros2")]
#[command(about = "Build tool for ROS 2 Rust projects", long_about = None)]
struct Ros2Args {
    #[command(subcommand)]
    command: Ros2Command,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Ros2Command {
    /// Build the project with ROS 2 bindings
    Build {
        /// Generate bindings only (don't run cargo build)
        #[arg(long)]
        bindings_only: bool,
    },

    /// Check the project with ROS 2 bindings
    Check {
        /// Generate bindings only (don't run cargo check)
        #[arg(long)]
        bindings_only: bool,
    },

    /// Clean generated bindings and cache
    Clean,
}

fn main() -> Result<()> {
    let CargoCli::Ros2(args) = CargoCli::parse();

    // Get project root (current directory)
    let project_root = env::current_dir()?;

    // Create workflow context
    let ctx = WorkflowContext::new(project_root, args.verbose);

    match args.command {
        Ros2Command::Build { bindings_only } => {
            ctx.run(bindings_only)?;
            if !bindings_only {
                println!("✓ Build complete!");
            } else {
                println!("✓ Bindings generated!");
            }
        }

        Ros2Command::Check { bindings_only } => {
            // For check, we run the same workflow but would invoke cargo check instead of build
            // For now, we just run the workflow
            ctx.run(bindings_only)?;
            if !bindings_only {
                println!("✓ Check complete!");
            } else {
                println!("✓ Bindings generated!");
            }
        }

        Ros2Command::Clean => {
            clean_bindings(&ctx)?;
            println!("✓ Cleaned bindings and cache!");
        }
    }

    Ok(())
}

fn clean_bindings(ctx: &WorkflowContext) -> Result<()> {
    // Remove bindings directory
    if ctx.output_dir.exists() {
        std::fs::remove_dir_all(&ctx.output_dir)?;
        if ctx.verbose {
            eprintln!("Removed {}", ctx.output_dir.display());
        }
    }

    // Remove cache file
    if ctx.cache_file.exists() {
        std::fs::remove_file(&ctx.cache_file)?;
        if ctx.verbose {
            eprintln!("Removed {}", ctx.cache_file.display());
        }
    }

    // Remove .cargo/config.toml patches (TODO: only remove ROS patches, not entire file)
    let cargo_config = ctx.project_root.join(".cargo").join("config.toml");
    if cargo_config.exists() && ctx.verbose {
        eprintln!("Note: .cargo/config.toml patches not removed (would need selective removal)");
    }

    Ok(())
}
