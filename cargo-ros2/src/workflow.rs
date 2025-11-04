//! Main workflow orchestration for cargo-ros2
//!
//! This module coordinates the entire process:
//! 1. Discover ROS dependencies from Cargo.toml
//! 2. Check cache for each package
//! 3. Generate missing/stale bindings
//! 4. Update cache
//! 5. Patch .cargo/config.toml
//! 6. Invoke cargo build

use crate::cache::{self, Cache, CacheEntry, CACHE_FILE_NAME};
use crate::config_patcher::ConfigPatcher;
use crate::dependency_parser::{DependencyParser, RosDependency};
use cargo_ros2_bindgen::ament::AmentIndex;
use eyre::{eyre, Result, WrapErr};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Workflow context
pub struct WorkflowContext {
    /// Project root directory
    pub project_root: PathBuf,
    /// Output directory for bindings (default: target/ros2_bindings)
    pub output_dir: PathBuf,
    /// Cache file path
    pub cache_file: PathBuf,
    /// Verbose output
    pub verbose: bool,
}

impl WorkflowContext {
    /// Create a new workflow context
    pub fn new(project_root: PathBuf, verbose: bool) -> Self {
        let output_dir = project_root.join("target").join("ros2_bindings");
        let cache_file = project_root.join(CACHE_FILE_NAME);

        WorkflowContext {
            project_root,
            output_dir,
            cache_file,
            verbose,
        }
    }

    /// Discover ROS dependencies via ament index
    pub fn discover_ament_packages(&self) -> Result<HashMap<String, PathBuf>> {
        let index =
            AmentIndex::from_env().wrap_err("Failed to load ament index (is ROS 2 sourced?)")?;

        let mut packages = HashMap::new();
        for (name, package) in index.packages() {
            packages.insert(name.clone(), package.share_dir.clone());
        }

        Ok(packages)
    }

    /// Discover ROS dependencies from Cargo.toml
    pub fn discover_ros_dependencies(&self) -> Result<Vec<RosDependency>> {
        // Get known ROS packages from ament index
        let ament_packages = self.discover_ament_packages()?;
        let known_ros_packages = ament_packages.keys().cloned().collect();

        // Parse Cargo.toml dependencies
        let parser = DependencyParser::new(known_ros_packages);
        parser.discover_dependencies(&self.project_root)
    }

    /// Check which packages need generation (cache miss or stale)
    pub fn check_cache(
        &self,
        dependencies: &[RosDependency],
        ament_packages: &HashMap<String, PathBuf>,
    ) -> Result<Vec<String>> {
        let cache = Cache::load(&self.cache_file)?;
        let mut to_generate = Vec::new();

        for dep in dependencies {
            // Get the package share dir
            let share_dir = match ament_packages.get(&dep.name) {
                Some(dir) => dir,
                None => {
                    // Package not in ament index, skip
                    continue;
                }
            };

            // Calculate current checksum
            let current_checksum = cache::calculate_package_checksum(share_dir)
                .wrap_err_with(|| format!("Failed to calculate checksum for {}", dep.name))?;

            // Check if cache is valid
            if !cache.is_valid(&dep.name, &current_checksum) {
                to_generate.push(dep.name.clone());
            }
        }

        Ok(to_generate)
    }

    /// Generate bindings for a package using cargo-ros2-bindgen
    pub fn generate_bindings(&self, package_name: &str) -> Result<PathBuf> {
        if self.verbose {
            eprintln!("  Generating bindings for {}...", package_name);
        }

        // Find cargo-ros2-bindgen binary
        let bindgen_binary = self.find_cargo_ros2_bindgen()?;

        // Build command
        let output_path = self.output_dir.clone();
        let mut cmd = Command::new(&bindgen_binary);
        cmd.arg("--package")
            .arg(package_name)
            .arg("--output")
            .arg(&output_path);

        if self.verbose {
            cmd.arg("--verbose");
        }

        // Execute
        let output = cmd
            .output()
            .wrap_err_with(|| format!("Failed to execute {}", bindgen_binary.display()))?;

        if !output.status.success() {
            return Err(eyre!(
                "cargo-ros2-bindgen failed for {}: {}",
                package_name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output_path.join(package_name))
    }

    /// Find cargo-ros2-bindgen binary
    fn find_cargo_ros2_bindgen(&self) -> Result<PathBuf> {
        // Try to find in target directory (development)
        let dev_path = self
            .project_root
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists())
            .map(|p| p.join("target").join("debug").join("cargo-ros2-bindgen"));

        if let Some(path) = dev_path {
            if path.exists() {
                return Ok(path);
            }
        }

        // Try to find in PATH
        if let Ok(output) = Command::new("which").arg("cargo-ros2-bindgen").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path_str));
            }
        }

        Err(eyre!(
            "cargo-ros2-bindgen not found. Please build it first with 'cargo build'"
        ))
    }

    /// Update cache after successful generation
    pub fn update_cache(
        &self,
        package_name: &str,
        package_share_dir: &PathBuf,
        output_dir: PathBuf,
    ) -> Result<()> {
        let mut cache = Cache::load(&self.cache_file)?;

        // Calculate checksum of the source package
        let checksum = cache::calculate_package_checksum(package_share_dir)
            .wrap_err_with(|| format!("Failed to calculate checksum for {}", package_name))?;

        let entry = CacheEntry {
            package_name: package_name.to_string(),
            checksum,
            ros_distro: std::env::var("ROS_DISTRO").ok(),
            package_version: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            output_dir,
        };

        cache.insert(entry);
        cache.save(&self.cache_file)?;

        Ok(())
    }

    /// Patch .cargo/config.toml with binding paths
    pub fn patch_cargo_config(&self, packages: &[(String, PathBuf)]) -> Result<()> {
        let mut patcher = ConfigPatcher::new(&self.project_root)?;

        for (package_name, package_path) in packages {
            if self.verbose {
                eprintln!(
                    "  Adding patch for {} -> {}",
                    package_name,
                    package_path.display()
                );
            }
            patcher.add_patch(package_name, package_path);
        }

        patcher.save()?;
        Ok(())
    }

    /// Run the complete workflow
    pub fn run(&self, bindings_only: bool) -> Result<()> {
        if self.verbose {
            eprintln!("cargo-ros2 workflow starting...");
        }

        // Step 1: Discover ament packages
        if self.verbose {
            eprintln!("Step 1: Discovering ROS packages from ament index...");
        }
        let ament_packages = self.discover_ament_packages()?;

        if self.verbose {
            eprintln!("  Found {} packages in ament index", ament_packages.len());
        }

        // Step 2: Discover ROS dependencies from Cargo.toml
        if self.verbose {
            eprintln!("Step 2: Discovering ROS dependencies from Cargo.toml...");
        }
        let dependencies = self.discover_ros_dependencies()?;

        if dependencies.is_empty() {
            eprintln!("No ROS 2 dependencies found in Cargo.toml");
            if !bindings_only {
                return self.invoke_cargo_build();
            }
            return Ok(());
        }

        if self.verbose {
            eprintln!("  Found {} ROS dependencies", dependencies.len());
        }

        // Step 3: Check cache
        if self.verbose {
            eprintln!("Step 3: Checking cache...");
        }
        let to_generate = self.check_cache(&dependencies, &ament_packages)?;

        if self.verbose {
            eprintln!("  {} packages need generation", to_generate.len());
        }

        // Step 4: Generate bindings
        let mut generated_packages = Vec::new();
        for package_name in &to_generate {
            let output_dir = self.generate_bindings(package_name)?;

            // Get share dir for checksum calculation
            if let Some(share_dir) = ament_packages.get(package_name) {
                self.update_cache(package_name, share_dir, output_dir.clone())?;
            }

            generated_packages.push((package_name.clone(), output_dir));
        }

        // Step 5: Patch .cargo/config.toml
        if !generated_packages.is_empty() {
            if self.verbose {
                eprintln!("Step 4: Patching .cargo/config.toml...");
            }
            self.patch_cargo_config(&generated_packages)?;
        }

        // Step 6: Invoke cargo build (unless --bindings-only)
        if !bindings_only {
            if self.verbose {
                eprintln!("Step 5: Invoking cargo build...");
            }
            self.invoke_cargo_build()?;
        }

        Ok(())
    }

    /// Invoke cargo build
    fn invoke_cargo_build(&self) -> Result<()> {
        if self.verbose {
            eprintln!("Step 4: Invoking cargo build...");
        }

        let status = Command::new("cargo")
            .arg("build")
            .current_dir(&self.project_root)
            .status()
            .wrap_err("Failed to execute cargo build")?;

        if !status.success() {
            return Err(eyre!("cargo build failed"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_context_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = WorkflowContext::new(temp_dir.path().to_path_buf(), false);

        assert_eq!(ctx.project_root, temp_dir.path());
        assert_eq!(
            ctx.output_dir,
            temp_dir.path().join("target").join("ros2_bindings")
        );
        assert_eq!(ctx.cache_file, temp_dir.path().join(CACHE_FILE_NAME));
        assert!(!ctx.verbose);
    }

    #[test]
    fn test_workflow_context_verbose() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = WorkflowContext::new(temp_dir.path().to_path_buf(), true);

        assert!(ctx.verbose);
    }

    #[test]
    fn test_discover_ament_packages_no_ros() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = WorkflowContext::new(temp_dir.path().to_path_buf(), false);

        // If ROS is not sourced, this will fail
        // If ROS is sourced, it should return packages
        let result = ctx.discover_ament_packages();

        // Either way is fine for this test - we're just checking it doesn't panic
        match result {
            Ok(packages) => {
                // ROS is sourced - packages may or may not be empty
                eprintln!("Found {} ROS packages", packages.len());
            }
            Err(e) => {
                // ROS is not sourced - expected
                // The error should mention the environment variable issue
                let error_str = e.to_string();
                assert!(
                    error_str.contains("AMENT_PREFIX_PATH")
                        || error_str.contains("environment variable not set")
                        || error_str.contains("Failed to load ament index"),
                    "Unexpected error message: {}",
                    error_str
                );
            }
        }
    }
}
