//! Main workflow orchestration for cargo-ros2
//!
//! This module coordinates the entire process:
//! 1. Discover ROS dependencies from Cargo.toml
//! 2. Check cache for each package
//! 3. Generate missing/stale bindings
//! 4. Update cache
//! 5. Patch .cargo/config.toml
//! 6. Invoke cargo build

use crate::cache::{Cache, CacheEntry, CACHE_FILE_NAME};
use crate::config_patcher::ConfigPatcher;
use crate::dependency_parser::{DependencyParser, RosDependency};
use eyre::{eyre, Result, WrapErr};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
        // This would use ament index, but for now we return empty
        // TODO: Integrate with cargo-ros2-bindgen's ament module
        Ok(HashMap::new())
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
    pub fn check_cache(&self, dependencies: &[RosDependency]) -> Result<Vec<String>> {
        let cache = Cache::load(&self.cache_file)?;
        let mut to_generate = Vec::new();

        for dep in dependencies {
            // For now, always generate if not in cache
            // TODO: Calculate checksums and validate
            if cache.get(&dep.name).is_none() {
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
    pub fn update_cache(&self, package_name: &str, output_dir: PathBuf) -> Result<()> {
        let mut cache = Cache::load(&self.cache_file)?;

        let entry = CacheEntry {
            package_name: package_name.to_string(),
            checksum: "TODO".to_string(), // TODO: Calculate actual checksum
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

        // Step 1: Discover ROS dependencies
        if self.verbose {
            eprintln!("Step 1: Discovering ROS dependencies...");
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

        // Step 2: Check cache
        if self.verbose {
            eprintln!("Step 2: Checking cache...");
        }
        let to_generate = self.check_cache(&dependencies)?;

        if self.verbose {
            eprintln!("  {} packages need generation", to_generate.len());
        }

        // Step 3: Generate bindings
        let mut generated_packages = Vec::new();
        for package_name in &to_generate {
            let output_dir = self.generate_bindings(package_name)?;
            self.update_cache(package_name, output_dir.clone())?;
            generated_packages.push((package_name.clone(), output_dir));
        }

        // Step 4: Patch .cargo/config.toml
        if !generated_packages.is_empty() {
            if self.verbose {
                eprintln!("Step 3: Patching .cargo/config.toml...");
            }
            self.patch_cargo_config(&generated_packages)?;
        }

        // Step 5: Invoke cargo build (unless --bindings-only)
        if !bindings_only {
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
    fn test_discover_ament_packages_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = WorkflowContext::new(temp_dir.path().to_path_buf(), false);

        let packages = ctx.discover_ament_packages().unwrap();
        assert!(packages.is_empty());
    }
}
