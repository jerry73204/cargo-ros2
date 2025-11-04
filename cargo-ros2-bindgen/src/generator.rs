//! Generator integration for generating Rust bindings from ROS 2 interface packages.
//!
//! This module integrates with rosidl-codegen to:
//! - Parse interface files (.msg, .srv, .action)
//! - Generate Rust code for messages, services, and actions
//! - Write generated code to output directory with proper structure

use crate::ament::Package;
use eyre::{Result, WrapErr};
use rosidl_codegen::{
    generate_action_package, generate_message_package, generate_service_package, GeneratedPackage,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Submodule name for C-compatible FFI layer (ROS Middleware layer).
/// This is the nested inline module that contains RMW (Raw Message Wire) format types.
///
/// The dual-layer architecture is:
/// - `pkg::msg::rmw::Type` - C-compatible FFI structs for interop with ROS C libraries
/// - `pkg::msg::Type` - Idiomatic Rust wrappers with safe types (String, Vec, etc.)
const RMW_SUBMODULE: &str = "rmw";

/// Generated Rust package structure
#[derive(Debug)]
pub struct GeneratedRustPackage {
    /// Package name
    pub name: String,
    /// Output directory where code was written
    pub output_dir: PathBuf,
    /// Number of messages generated
    pub message_count: usize,
    /// Number of services generated
    pub service_count: usize,
    /// Number of actions generated
    pub action_count: usize,
}

/// Generate Rust bindings for a ROS 2 package
pub fn generate_package(package: &Package, output_dir: &Path) -> Result<GeneratedRustPackage> {
    let package_output = output_dir.join(&package.name);
    std::fs::create_dir_all(&package_output).wrap_err_with(|| {
        format!(
            "Failed to create output directory: {}",
            package_output.display()
        )
    })?;

    let mut message_count = 0;
    let mut service_count = 0;
    let mut action_count = 0;

    // For dependency tracking (cross-package references)
    let known_packages = HashSet::new(); // TODO: populate from ament index

    // Generate messages
    for msg_name in &package.interfaces.messages {
        let msg_path = package.get_message_path(msg_name);
        let content = std::fs::read_to_string(&msg_path)
            .wrap_err_with(|| format!("Failed to read message file: {}", msg_path.display()))?;

        let parsed_msg = rosidl_parser::parse_message(&content)
            .wrap_err_with(|| format!("Failed to parse message: {}", msg_name))?;

        let generated =
            generate_message_package(&package.name, msg_name, &parsed_msg, &known_packages)
                .wrap_err_with(|| format!("Failed to generate message: {}", msg_name))?;

        write_generated_package(&generated, &package_output, msg_name)?;
        message_count += 1;
    }

    // Generate services
    for srv_name in &package.interfaces.services {
        let srv_path = package.get_service_path(srv_name);
        let content = std::fs::read_to_string(&srv_path)
            .wrap_err_with(|| format!("Failed to read service file: {}", srv_path.display()))?;

        let parsed_srv = rosidl_parser::parse_service(&content)
            .wrap_err_with(|| format!("Failed to parse service: {}", srv_name))?;

        let generated =
            generate_service_package(&package.name, srv_name, &parsed_srv, &known_packages)
                .wrap_err_with(|| format!("Failed to generate service: {}", srv_name))?;

        write_generated_service(&generated, &package_output, srv_name)?;
        service_count += 1;
    }

    // Generate actions
    for action_name in &package.interfaces.actions {
        let action_path = package.get_action_path(action_name);
        let content = std::fs::read_to_string(&action_path)
            .wrap_err_with(|| format!("Failed to read action file: {}", action_path.display()))?;

        let parsed_action = rosidl_parser::parse_action(&content)
            .wrap_err_with(|| format!("Failed to parse action: {}", action_name))?;

        let generated =
            generate_action_package(&package.name, action_name, &parsed_action, &known_packages)
                .wrap_err_with(|| format!("Failed to generate action: {}", action_name))?;

        write_generated_action(&generated, &package_output, action_name)?;
        action_count += 1;
    }

    // Generate lib.rs that re-exports all generated code
    generate_lib_rs(&package_output, package)?;

    // Generate Cargo.toml for the package
    generate_cargo_toml(&package_output, &package.name)?;

    // Generate build.rs for FFI linking
    generate_build_rs(&package_output, &package.name)?;

    Ok(GeneratedRustPackage {
        name: package.name.clone(),
        output_dir: package_output,
        message_count,
        service_count,
        action_count,
    })
}

/// Write generated message package to files
fn write_generated_package(
    generated: &GeneratedPackage,
    output_dir: &Path,
    name: &str,
) -> Result<()> {
    let msg_dir = output_dir.join("src").join("msg");
    std::fs::create_dir_all(&msg_dir)?;

    // Create RMW subdirectory for nested inline module
    let rmw_dir = msg_dir.join(RMW_SUBMODULE);
    std::fs::create_dir_all(&rmw_dir)?;

    // Write RMW message to msg/rmw/ subdirectory
    let rmw_file = rmw_dir.join(format!("{}_rmw.rs", name.to_lowercase()));
    std::fs::write(&rmw_file, &generated.message_rmw)?;

    // Write idiomatic message to msg/ directory
    let idiomatic_file = msg_dir.join(format!("{}_idiomatic.rs", name.to_lowercase()));
    std::fs::write(&idiomatic_file, &generated.message_idiomatic)?;

    Ok(())
}

/// Write generated service package to files
fn write_generated_service(
    generated: &rosidl_codegen::GeneratedServicePackage,
    output_dir: &Path,
    name: &str,
) -> Result<()> {
    let srv_dir = output_dir.join("src").join("srv");
    std::fs::create_dir_all(&srv_dir)?;

    // Create RMW subdirectory for nested inline module
    let rmw_dir = srv_dir.join(RMW_SUBMODULE);
    std::fs::create_dir_all(&rmw_dir)?;

    // Write RMW service to srv/rmw/ subdirectory
    let rmw_file = rmw_dir.join(format!("{}_rmw.rs", name.to_lowercase()));
    std::fs::write(&rmw_file, &generated.service_rmw)?;

    // Write idiomatic service to srv/ directory
    let idiomatic_file = srv_dir.join(format!("{}_idiomatic.rs", name.to_lowercase()));
    std::fs::write(&idiomatic_file, &generated.service_idiomatic)?;

    Ok(())
}

/// Write generated action package to files
fn write_generated_action(
    generated: &rosidl_codegen::GeneratedActionPackage,
    output_dir: &Path,
    name: &str,
) -> Result<()> {
    let action_dir = output_dir.join("src").join("action");
    std::fs::create_dir_all(&action_dir)?;

    // Create RMW subdirectory for nested inline module
    let rmw_dir = action_dir.join(RMW_SUBMODULE);
    std::fs::create_dir_all(&rmw_dir)?;

    // Write RMW action to action/rmw/ subdirectory
    let rmw_file = rmw_dir.join(format!("{}_rmw.rs", name.to_lowercase()));
    std::fs::write(&rmw_file, &generated.action_rmw)?;

    // Write idiomatic action to action/ directory
    let idiomatic_file = action_dir.join(format!("{}_idiomatic.rs", name.to_lowercase()));
    std::fs::write(&idiomatic_file, &generated.action_idiomatic)?;

    Ok(())
}

/// Generate lib.rs that re-exports all generated modules
fn generate_lib_rs(output_dir: &Path, package: &Package) -> Result<()> {
    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let mut lib_rs = String::new();
    lib_rs.push_str("// Auto-generated Rust bindings for ROS 2 interface package\n");
    lib_rs.push_str(&format!("// Package: {}\n\n", package.name));

    // Add rosidl_runtime_rs stub (for now - will be real dependency later)
    lib_rs.push_str("// TODO: Use real rosidl_runtime_rs crate\n");
    lib_rs.push_str("pub mod rosidl_runtime_rs {\n");
    lib_rs.push_str("    pub trait SequenceAlloc {}\n");
    lib_rs.push_str("    pub trait Message {}\n");
    lib_rs.push_str("    pub trait RmwMessage {}\n");
    lib_rs.push_str("    pub trait Service {}\n");
    lib_rs.push_str("    pub trait Action {}\n");
    lib_rs.push_str("    #[repr(C)]\n");
    lib_rs.push_str("    pub struct Sequence<T> { _phantom: std::marker::PhantomData<T> }\n");
    lib_rs.push_str("}\n\n");

    // Add message modules
    if !package.interfaces.messages.is_empty() {
        lib_rs.push_str("pub mod msg {\n");
        lib_rs.push_str("    use super::rosidl_runtime_rs;\n\n");
        lib_rs.push_str(&format!("    pub mod {} {{\n", RMW_SUBMODULE));
        lib_rs.push_str("        use super::*;\n");
        for msg_name in &package.interfaces.messages {
            let module_name = msg_name.to_lowercase();
            // Files are in src/msg/{RMW_SUBMODULE}/, inline module context is also msg/{RMW_SUBMODULE}/
            lib_rs.push_str(&format!("        #[path = \"{}_rmw.rs\"]\n", module_name));
            lib_rs.push_str(&format!("        pub mod {};\n", module_name));
        }
        lib_rs.push_str("    }\n\n");
        for msg_name in &package.interfaces.messages {
            let module_name = msg_name.to_lowercase();
            // Files are in src/msg/, inline module context is also msg/
            lib_rs.push_str(&format!("    #[path = \"{}_idiomatic.rs\"]\n", module_name));
            lib_rs.push_str(&format!("    pub mod {};\n", module_name));
        }
        lib_rs.push_str("}\n\n");
    }

    // Add service modules
    if !package.interfaces.services.is_empty() {
        lib_rs.push_str("pub mod srv {\n");
        lib_rs.push_str("    use super::rosidl_runtime_rs;\n\n");
        lib_rs.push_str(&format!("    pub mod {} {{\n", RMW_SUBMODULE));
        lib_rs.push_str("        use super::*;\n");
        for srv_name in &package.interfaces.services {
            let module_name = srv_name.to_lowercase();
            // Files are in src/srv/{RMW_SUBMODULE}/, inline module context is also srv/{RMW_SUBMODULE}/
            lib_rs.push_str(&format!("        #[path = \"{}_rmw.rs\"]\n", module_name));
            lib_rs.push_str(&format!("        pub mod {};\n", module_name));
        }
        lib_rs.push_str("    }\n\n");
        for srv_name in &package.interfaces.services {
            let module_name = srv_name.to_lowercase();
            // Files are in src/srv/, inline module context is also srv/
            lib_rs.push_str(&format!("    #[path = \"{}_idiomatic.rs\"]\n", module_name));
            lib_rs.push_str(&format!("    pub mod {};\n", module_name));
        }
        lib_rs.push_str("}\n\n");
    }

    // Add action modules
    if !package.interfaces.actions.is_empty() {
        lib_rs.push_str("pub mod action {\n");
        lib_rs.push_str("    use super::rosidl_runtime_rs;\n\n");
        lib_rs.push_str(&format!("    pub mod {} {{\n", RMW_SUBMODULE));
        lib_rs.push_str("        use super::*;\n");
        for action_name in &package.interfaces.actions {
            let module_name = action_name.to_lowercase();
            // Files are in src/action/{RMW_SUBMODULE}/, inline module context is also action/{RMW_SUBMODULE}/
            lib_rs.push_str(&format!("        #[path = \"{}_rmw.rs\"]\n", module_name));
            lib_rs.push_str(&format!("        pub mod {};\n", module_name));
        }
        lib_rs.push_str("    }\n\n");
        for action_name in &package.interfaces.actions {
            let module_name = action_name.to_lowercase();
            // Files are in src/action/, inline module context is also action/
            lib_rs.push_str(&format!("    #[path = \"{}_idiomatic.rs\"]\n", module_name));
            lib_rs.push_str(&format!("    pub mod {};\n", module_name));
        }
        lib_rs.push_str("}\n");
    }

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

/// Generate Cargo.toml for the generated package
fn generate_cargo_toml(output_dir: &Path, package_name: &str) -> Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

# Standalone package (not part of parent workspace)
[workspace]

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}

[build-dependencies]
# For linking against ROS 2 C libraries
"#,
        package_name
    );

    std::fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

/// Generate build.rs for linking against ROS 2 C libraries
fn generate_build_rs(output_dir: &Path, package_name: &str) -> Result<()> {
    let build_rs = format!(
        r#"fn main() {{
    // Link against ROS 2 C libraries
    println!("cargo:rustc-link-lib={package}__rosidl_typesupport_c");
    println!("cargo:rustc-link-lib={package}__rosidl_generator_c");
}}
"#,
        package = package_name
    );

    std::fs::write(output_dir.join("build.rs"), build_rs)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ament::Package;
    use std::fs;

    /// Helper to create a test package with interface files
    fn create_test_package(temp_dir: &Path) -> Package {
        let share_dir = temp_dir.join("test_pkg");

        // Create msg files
        let msg_dir = share_dir.join("msg");
        fs::create_dir_all(&msg_dir).unwrap();
        fs::write(msg_dir.join("Point.msg"), "float64 x\nfloat64 y\n").unwrap();

        // Create srv files
        let srv_dir = share_dir.join("srv");
        fs::create_dir_all(&srv_dir).unwrap();
        fs::write(
            srv_dir.join("AddTwoInts.srv"),
            "int64 a\nint64 b\n---\nint64 sum\n",
        )
        .unwrap();

        // Create action files
        let action_dir = share_dir.join("action");
        fs::create_dir_all(&action_dir).unwrap();
        fs::write(
            action_dir.join("Fibonacci.action"),
            "int32 order\n---\nint32[] sequence\n---\nint32[] partial_sequence\n",
        )
        .unwrap();

        Package::from_share_dir(share_dir).unwrap()
    }

    #[test]
    fn test_generate_message() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package = create_test_package(temp_dir.path());
        let output_dir = temp_dir.path().join("output");

        let result = generate_package(&package, &output_dir);
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert_eq!(generated.message_count, 1);
        assert_eq!(generated.service_count, 1);
        assert_eq!(generated.action_count, 1);

        // Check that files were created
        let pkg_dir = output_dir.join("test_pkg");
        assert!(pkg_dir.join("Cargo.toml").exists());
        assert!(pkg_dir.join("build.rs").exists());
        assert!(pkg_dir.join("src").join("lib.rs").exists());
    }

    #[test]
    fn test_generate_lib_rs_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package = create_test_package(temp_dir.path());
        let output_dir = temp_dir.path().join("output");
        std::fs::create_dir_all(&output_dir).unwrap();

        generate_lib_rs(&output_dir, &package).unwrap();

        let lib_rs_content =
            std::fs::read_to_string(output_dir.join("src").join("lib.rs")).unwrap();
        assert!(lib_rs_content.contains("pub mod msg"));
        assert!(lib_rs_content.contains("pub mod srv"));
        assert!(lib_rs_content.contains("pub mod action"));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        generate_cargo_toml(temp_dir.path(), "test_pkg").unwrap();

        let cargo_toml = std::fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("name = \"test_pkg\""));
        assert!(cargo_toml.contains("serde"));
    }

    #[test]
    fn test_build_rs_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        generate_build_rs(temp_dir.path(), "test_pkg").unwrap();

        let build_rs = std::fs::read_to_string(temp_dir.path().join("build.rs")).unwrap();
        assert!(build_rs.contains("test_pkg__rosidl_typesupport_c"));
        assert!(build_rs.contains("test_pkg__rosidl_generator_c"));
    }

    #[test]
    fn test_invalid_message_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let share_dir = temp_dir.path().join("bad_pkg");
        let msg_dir = share_dir.join("msg");
        fs::create_dir_all(&msg_dir).unwrap();
        fs::write(msg_dir.join("Bad.msg"), "invalid syntax here!!! @#$%\n").unwrap();

        let package = Package::from_share_dir(share_dir).unwrap();
        let output_dir = temp_dir.path().join("output");

        let result = generate_package(&package, &output_dir);
        assert!(result.is_err());
    }
}
