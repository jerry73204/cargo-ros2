//! Ament installation support for cargo-ros2
//!
//! This module handles installing Rust packages to the ament index structure,
//! similar to cargo-ament-build. It creates the necessary markers, installs
//! source files, binaries, and metadata.

use eyre::{Result, WrapErr};
use std::fs;
use std::path::{Path, PathBuf};

/// Ament installer for creating ament-compatible installations
pub struct AmentInstaller {
    /// Install base directory (e.g., install/package_name)
    install_base: PathBuf,
    /// Package name
    package_name: String,
    /// Project root directory
    project_root: PathBuf,
    /// Verbose output
    verbose: bool,
    /// Build profile (debug or release)
    profile: String,
}

impl AmentInstaller {
    /// Create a new ament installer
    pub fn new(
        install_base: PathBuf,
        package_name: String,
        project_root: PathBuf,
        verbose: bool,
        profile: String,
    ) -> Self {
        Self {
            install_base,
            package_name,
            project_root,
            verbose,
            profile,
        }
    }

    /// Run the complete installation process
    pub fn install(&self, is_library: bool) -> Result<()> {
        if self.verbose {
            eprintln!(
                "Installing {} to {}",
                self.package_name,
                self.install_base.display()
            );
        }

        // Create directory structure
        self.create_directories()?;

        // Create ament index markers
        self.create_markers()?;

        // Install source files
        self.install_source_files()?;

        // Install binaries (if not a pure library)
        if !is_library {
            self.install_binaries()?;
        }

        // Install metadata
        self.install_metadata()?;

        if self.verbose {
            eprintln!("✓ Installation complete!");
        }

        Ok(())
    }

    /// Create necessary directory structure
    fn create_directories(&self) -> Result<()> {
        let dirs = [
            self.lib_dir(),
            self.share_dir(),
            self.ament_index_dir(),
            self.rust_source_dir(),
        ];

        for dir in &dirs {
            fs::create_dir_all(dir)
                .wrap_err_with(|| format!("Failed to create directory: {}", dir.display()))?;
        }

        Ok(())
    }

    /// Create ament index markers
    fn create_markers(&self) -> Result<()> {
        // Create package marker
        let marker_file = self
            .ament_index_dir()
            .join("resource_index")
            .join("packages")
            .join(&self.package_name);

        fs::create_dir_all(marker_file.parent().unwrap())?;
        fs::write(&marker_file, "")?;

        if self.verbose {
            eprintln!("  Created marker: {}", marker_file.display());
        }

        // Create package type marker (Rust)
        let package_type_file = self
            .ament_index_dir()
            .join("resource_index")
            .join("package_type")
            .join(&self.package_name);

        fs::create_dir_all(package_type_file.parent().unwrap())?;
        fs::write(&package_type_file, "rust")?;

        if self.verbose {
            eprintln!(
                "  Created package type marker: {}",
                package_type_file.display()
            );
        }

        Ok(())
    }

    /// Install source files to share directory
    fn install_source_files(&self) -> Result<()> {
        let source_files = [("Cargo.toml", false), ("Cargo.lock", false), ("src", true)];

        let dest_dir = self.rust_source_dir();

        for (name, is_dir) in &source_files {
            let source = self.project_root.join(name);
            let dest = dest_dir.join(name);

            if !source.exists() {
                continue;
            }

            if *is_dir {
                self.copy_dir_recursive(&source, &dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&source, &dest).wrap_err_with(|| {
                    format!("Failed to copy {} to {}", source.display(), dest.display())
                })?;
            }

            if self.verbose {
                eprintln!("  Installed: {}", name);
            }
        }

        Ok(())
    }

    /// Install binaries to lib directory
    fn install_binaries(&self) -> Result<()> {
        let target_dir = self.project_root.join("target").join(&self.profile);
        let cargo_toml_path = self.project_root.join("Cargo.toml");

        // Parse Cargo.toml to find binary names
        let cargo_toml =
            fs::read_to_string(&cargo_toml_path).wrap_err("Failed to read Cargo.toml")?;

        let binaries = self.extract_binary_names(&cargo_toml);

        if binaries.is_empty() {
            if self.verbose {
                eprintln!("  No binaries to install (library package)");
            }
            return Ok(());
        }

        let dest_dir = self.lib_dir().join(&self.package_name);
        fs::create_dir_all(&dest_dir)?;

        for binary_name in binaries {
            let source = target_dir.join(&binary_name);
            let dest = dest_dir.join(&binary_name);

            if source.exists() {
                fs::copy(&source, &dest)
                    .wrap_err_with(|| format!("Failed to copy binary: {}", binary_name))?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&dest)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest, perms)?;
                }

                if self.verbose {
                    eprintln!("  Installed binary: {}", binary_name);
                }
            } else if self.verbose {
                eprintln!(
                    "  Warning: Binary not found: {} (did you run with --release?)",
                    binary_name
                );
            }
        }

        Ok(())
    }

    /// Install metadata files
    fn install_metadata(&self) -> Result<()> {
        let package_xml_source = self.project_root.join("package.xml");
        let package_xml_dest = self.share_dir().join("package.xml");

        if package_xml_source.exists() {
            fs::copy(&package_xml_source, &package_xml_dest)
                .wrap_err("Failed to copy package.xml")?;

            if self.verbose {
                eprintln!("  Installed: package.xml");
            }
        } else if self.verbose {
            eprintln!("  Note: No package.xml found (optional)");
        }

        Ok(())
    }

    /// Extract binary names from Cargo.toml
    fn extract_binary_names(&self, cargo_toml: &str) -> Vec<String> {
        let mut binaries = Vec::new();

        // Simple parser for [[bin]] sections
        let mut in_bin_section = false;

        for line in cargo_toml.lines() {
            let trimmed = line.trim();

            if trimmed == "[[bin]]" {
                in_bin_section = true;
                continue;
            }

            if in_bin_section {
                if trimmed.starts_with('[') {
                    in_bin_section = false;
                    continue;
                }

                if trimmed.starts_with("name") {
                    if let Some(name) = self.extract_toml_string_value(trimmed) {
                        binaries.push(name);
                    }
                }
            }
        }

        // Also check for default binary (package name)
        if binaries.is_empty() {
            binaries.push(self.package_name.replace('-', "_"));
        }

        binaries
    }

    /// Extract string value from TOML line (simple parser)
    fn extract_toml_string_value(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1].trim();
        let value = value.trim_matches('"').trim_matches('\'');
        Some(value.to_string())
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        copy_dir_recursive_impl(src, dst)
    }

    /// Get lib directory path
    fn lib_dir(&self) -> PathBuf {
        self.install_base.join("lib")
    }

    /// Get share directory path
    fn share_dir(&self) -> PathBuf {
        self.install_base.join("share").join(&self.package_name)
    }

    /// Get ament index directory path
    fn ament_index_dir(&self) -> PathBuf {
        self.share_dir().join("ament_index")
    }

    /// Get rust source directory path
    fn rust_source_dir(&self) -> PathBuf {
        self.share_dir().join("rust")
    }
}

/// Copy directory recursively (helper function)
fn copy_dir_recursive_impl(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive_impl(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Check if a package is a pure library (no binaries)
pub fn is_library_package(project_root: &Path) -> Result<bool> {
    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path).wrap_err("Failed to read Cargo.toml")?;

    // Check if there's a [[bin]] section or default binary
    let has_bin_section = cargo_toml.contains("[[bin]]");
    let has_default_main = project_root.join("src").join("main.rs").exists();

    Ok(!has_bin_section && !has_default_main)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ament_installer_directories() {
        let temp_dir = TempDir::new().unwrap();
        let install_base = temp_dir.path().join("install").join("test_pkg");
        let project_root = temp_dir.path().join("project");

        let installer = AmentInstaller::new(
            install_base.clone(),
            "test_pkg".to_string(),
            project_root,
            false,
            "debug".to_string(),
        );

        assert_eq!(installer.lib_dir(), install_base.join("lib"));
        assert_eq!(
            installer.share_dir(),
            install_base.join("share").join("test_pkg")
        );
        assert_eq!(
            installer.ament_index_dir(),
            install_base
                .join("share")
                .join("test_pkg")
                .join("ament_index")
        );
        assert_eq!(
            installer.rust_source_dir(),
            install_base.join("share").join("test_pkg").join("rust")
        );
    }

    #[test]
    fn test_is_library_package() {
        let temp_dir = TempDir::new().unwrap();

        // Create a library package
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "test_lib"
"#,
        )
        .unwrap();

        fs::write(temp_dir.path().join("src").join("lib.rs"), "").unwrap();

        assert!(is_library_package(temp_dir.path()).unwrap());
    }

    #[test]
    fn test_is_not_library_package() {
        let temp_dir = TempDir::new().unwrap();

        // Create a binary package
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test-bin"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::write(temp_dir.path().join("src").join("main.rs"), "fn main() {}").unwrap();

        assert!(!is_library_package(temp_dir.path()).unwrap());
    }

    #[test]
    fn test_extract_binary_names() {
        let temp_dir = TempDir::new().unwrap();
        let installer = AmentInstaller::new(
            temp_dir.path().to_path_buf(),
            "my-pkg".to_string(),
            temp_dir.path().to_path_buf(),
            false,
            "debug".to_string(),
        );

        let cargo_toml = r#"
[package]
name = "my-pkg"

[[bin]]
name = "my-binary"
path = "src/main.rs"

[[bin]]
name = "other-binary"
path = "src/other.rs"
"#;

        let binaries = installer.extract_binary_names(cargo_toml);
        assert_eq!(binaries.len(), 2);
        assert!(binaries.contains(&"my-binary".to_string()));
        assert!(binaries.contains(&"other-binary".to_string()));
    }

    #[test]
    fn test_extract_toml_string_value() {
        let temp_dir = TempDir::new().unwrap();
        let installer = AmentInstaller::new(
            temp_dir.path().to_path_buf(),
            "test".to_string(),
            temp_dir.path().to_path_buf(),
            false,
            "debug".to_string(),
        );

        assert_eq!(
            installer.extract_toml_string_value("name = \"my-binary\""),
            Some("my-binary".to_string())
        );

        assert_eq!(
            installer.extract_toml_string_value("name='other'"),
            Some("other".to_string())
        );

        assert_eq!(installer.extract_toml_string_value("invalid"), None);
    }
}
