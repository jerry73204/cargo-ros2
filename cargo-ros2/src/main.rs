use cargo_ros2::workflow::WorkflowContext;
use clap::{Parser, Subcommand};
use eyre::{eyre, Result, WrapErr};
use std::env;
use std::path::{Path, PathBuf};

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

    /// Cache management commands
    Cache {
        #[command(subcommand)]
        cache_command: CacheCommand,
    },

    /// Show information about a ROS 2 package
    Info {
        /// Package name to show information about
        package: String,
    },

    /// Build and install package to ament index (colcon-compatible)
    AmentBuild {
        /// Install base directory
        #[arg(long)]
        install_base: PathBuf,

        /// Build with release profile
        #[arg(long)]
        release: bool,

        /// Look up dependencies in the workspace directory
        #[arg(long)]
        lookup_in_workspace: bool,

        /// Additional arguments to pass to cargo build
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    /// List cached package bindings
    List,

    /// Rebuild bindings for a specific package
    Rebuild {
        /// Package name to rebuild
        package: String,
    },

    /// Clean all cached bindings
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

        Ros2Command::Cache { cache_command } => {
            handle_cache_command(&ctx, &cache_command)?;
        }

        Ros2Command::Info { package } => {
            show_package_info(&ctx, &package)?;
        }

        Ros2Command::AmentBuild {
            install_base,
            release,
            lookup_in_workspace,
            cargo_args,
        } => {
            ament_build(
                &ctx,
                &install_base,
                release,
                lookup_in_workspace,
                &cargo_args,
            )?;
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

fn handle_cache_command(ctx: &WorkflowContext, command: &CacheCommand) -> Result<()> {
    use cargo_ros2::cache::Cache;

    match command {
        CacheCommand::List => {
            let cache = Cache::load(&ctx.cache_file)?;

            if cache.is_empty() {
                println!("No cached bindings found.");
                return Ok(());
            }

            println!("Cached ROS 2 package bindings:\n");
            println!(
                "{:<30} {:<15} {:<12} {:<50}",
                "Package", "ROS Distro", "Checksum", "Output Directory"
            );
            println!("{}", "-".repeat(100));

            let mut entries: Vec<_> = cache.entries().collect();
            entries.sort_by_key(|e| &e.package_name);

            for entry in entries {
                let distro = entry.ros_distro.as_deref().unwrap_or("unknown");
                let checksum_short = if entry.checksum.len() > 12 {
                    format!("{}...", &entry.checksum[..9])
                } else {
                    entry.checksum.clone()
                };

                println!(
                    "{:<30} {:<15} {:<12} {}",
                    entry.package_name,
                    distro,
                    checksum_short,
                    entry.output_dir.display()
                );
            }

            println!("\nTotal: {} package(s)", cache.len());
        }

        CacheCommand::Rebuild { package } => {
            let mut cache = Cache::load(&ctx.cache_file)?;

            // Remove from cache
            cache.remove(package);
            cache.save(&ctx.cache_file)?;

            println!(
                "Removed {} from cache. Run 'cargo ros2 build' to regenerate.",
                package
            );
        }

        CacheCommand::Clean => {
            clean_bindings(ctx)?;
            println!("✓ Cache cleaned!");
        }
    }

    Ok(())
}

fn show_package_info(ctx: &WorkflowContext, package_name: &str) -> Result<()> {
    use cargo_ros2_bindgen::ament::AmentIndex;
    use eyre::eyre;

    // Load ament index
    let index = AmentIndex::from_env()
        .map_err(|_| eyre!("Failed to load ament index. Is ROS 2 sourced?"))?;

    // Find package
    let package = index
        .find_package(package_name)
        .ok_or_else(|| eyre!("Package '{}' not found in ament index", package_name))?;

    println!("Package: {}", package.name);
    println!("Share directory: {}", package.share_dir.display());
    println!();

    println!("Interfaces:");
    if !package.interfaces.messages.is_empty() {
        println!("  Messages ({}):", package.interfaces.messages.len());
        for msg in &package.interfaces.messages {
            println!("    - {}", msg);
        }
    }

    if !package.interfaces.services.is_empty() {
        println!("  Services ({}):", package.interfaces.services.len());
        for srv in &package.interfaces.services {
            println!("    - {}", srv);
        }
    }

    if !package.interfaces.actions.is_empty() {
        println!("  Actions ({}):", package.interfaces.actions.len());
        for action in &package.interfaces.actions {
            println!("    - {}", action);
        }
    }

    if package.interfaces.messages.is_empty()
        && package.interfaces.services.is_empty()
        && package.interfaces.actions.is_empty()
    {
        println!("  No interface files found");
    }

    println!();

    // Check if cached
    use cargo_ros2::cache::Cache;
    let cache = Cache::load(&ctx.cache_file)?;

    if let Some(entry) = cache.get(package_name) {
        println!("Cache status: ✓ Cached");
        println!("  Checksum: {}", entry.checksum);
        println!("  Output: {}", entry.output_dir.display());
        if let Some(distro) = &entry.ros_distro {
            println!("  ROS Distro: {}", distro);
        }
    } else {
        println!("Cache status: Not cached");
    }

    Ok(())
}

fn ament_build(
    ctx: &WorkflowContext,
    install_base: &Path,
    release: bool,
    lookup_in_workspace: bool,
    cargo_args: &[String],
) -> Result<()> {
    use cargo_ros2::ament_installer::{is_library_package, AmentInstaller};
    use cargo_ros2::package_discovery::{
        discover_installed_ament_packages, discover_workspace_packages,
    };
    use std::collections::HashMap;
    use std::process::Command;

    println!("Building and installing package to ament index...");

    // Step 1: Collect all patches BEFORE generating bindings
    if ctx.verbose {
        eprintln!("Step 1: Collecting package patches...");
    }

    let mut all_patches: HashMap<String, PathBuf> = HashMap::new();

    // 1a. Workspace packages (if --lookup-in-workspace)
    if lookup_in_workspace {
        if ctx.verbose {
            eprintln!("  Discovering workspace packages...");
        }

        // Find workspace root (go up from project_root until we find no parent or hit root)
        let mut workspace_root = ctx.project_root.clone();
        while let Some(parent) = workspace_root.parent() {
            // Check if parent looks like a workspace (has build/ or install/)
            if parent.join("build").exists() || parent.join("install").exists() {
                workspace_root = parent.to_path_buf();
            } else {
                break;
            }
        }

        let build_base = workspace_root.join("build");
        let install_base_abs = if install_base.is_absolute() {
            install_base.to_path_buf()
        } else {
            workspace_root.join(install_base)
        };

        let workspace_pkgs = discover_workspace_packages(
            &workspace_root,
            Some(&build_base),
            Some(&install_base_abs),
        )?;

        if ctx.verbose {
            eprintln!("    Found {} workspace packages", workspace_pkgs.len());
        }

        all_patches.extend(workspace_pkgs);
    }

    // 1b. Installed ament packages
    if ctx.verbose {
        eprintln!("  Discovering installed ament packages...");
    }

    let installed_pkgs = discover_installed_ament_packages()?;

    if ctx.verbose {
        eprintln!("    Found {} installed packages", installed_pkgs.len());
    }

    all_patches.extend(installed_pkgs);

    // Step 2: Generate bindings
    if ctx.verbose {
        eprintln!("Step 2: Generating ROS 2 bindings...");
    }

    ctx.run(true)?; // bindings_only = true

    // 2b. Add generated bindings to patches
    // Generated bindings are in target/ros2_bindings/<package>/
    let bindings_dir = ctx.output_dir.clone();
    if bindings_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&bindings_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let package_name = entry.file_name().to_string_lossy().to_string();
                    all_patches.insert(package_name, entry.path());
                }
            }
        }
    }

    // Step 3: Write unified config.toml (SINGLE WRITE)
    if ctx.verbose {
        eprintln!("Step 3: Writing unified .cargo/config.toml...");
        eprintln!("  Total patches: {}", all_patches.len());
    }

    let patches: Vec<(String, PathBuf)> = all_patches.into_iter().collect();
    ctx.patch_cargo_config(&patches)?;

    // Step 4: Build the package
    if ctx.verbose {
        eprintln!(
            "Step 4: Building package{}...",
            if release { " (release)" } else { "" }
        );
    }

    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build").current_dir(&ctx.project_root);

    if release {
        build_cmd.arg("--release");
    }

    // Add additional cargo arguments
    for arg in cargo_args {
        build_cmd.arg(arg);
    }

    let status = build_cmd
        .status()
        .wrap_err("Failed to execute cargo build")?;

    if !status.success() {
        return Err(eyre::eyre!("cargo build failed"));
    }

    // Step 5: Get package name from Cargo.toml
    let cargo_toml_path = ctx.project_root.join("Cargo.toml");
    let cargo_toml =
        std::fs::read_to_string(&cargo_toml_path).wrap_err("Failed to read Cargo.toml")?;

    let package_name = extract_package_name(&cargo_toml)
        .ok_or_else(|| eyre::eyre!("Failed to extract package name from Cargo.toml"))?;

    // Step 6: Check if it's a library package
    let is_library = is_library_package(&ctx.project_root)?;

    if ctx.verbose {
        eprintln!(
            "Step 5: Installing {} package...",
            if is_library { "library" } else { "binary" }
        );
    }

    // Step 7: Install using ament installer
    let package_install_base = install_base.join(&package_name);
    let installer = AmentInstaller::new(
        package_install_base.clone(),
        package_name.clone(),
        ctx.project_root.clone(),
        ctx.verbose,
    );

    installer.install(is_library)?;

    println!("✓ Installation complete!");
    println!("  Install location: {}", package_install_base.display());
    println!("  Package name: {}", package_name);
    println!("  Type: {}", if is_library { "library" } else { "binary" });

    Ok(())
}

fn extract_package_name(cargo_toml: &str) -> Option<String> {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = &trimmed[eq_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}
