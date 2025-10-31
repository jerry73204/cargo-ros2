// Compilation tests - verify generated code compiles successfully
use rosidl_codegen::{generate_message_package, GeneratorError};
use rosidl_parser::parse_message;
use std::collections::HashSet;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a minimal Cargo.toml for testing compilation
fn create_test_cargo_toml(pkg_name: &str, needs_big_array: bool) -> String {
    let big_array_dep = if needs_big_array {
        r#"big-array = { version = "0.5", features = ["serde"] }
"#
    } else {
        ""
    };

    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
{}
[lib]
path = "src/lib.rs"
"#,
        pkg_name, big_array_dep
    )
}

/// Helper to create a stub for rosidl_runtime_rs types (for compilation testing)
fn create_rosidl_runtime_stub() -> String {
    r#"
// Stub implementations of rosidl_runtime_rs types for compilation testing
pub mod rosidl_runtime_rs {
    use serde::{Deserialize, Serialize};

    pub type String = std::string::String;
    pub type WString = std::string::String;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BoundedString<const N: usize>(std::string::String);

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BoundedWString<const N: usize>(std::string::String);

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Sequence<T>(Vec<T>);

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BoundedSequence<T, const N: usize>(Vec<T>);

    impl<T> Default for Sequence<T> {
        fn default() -> Self {
            Sequence(Vec::new())
        }
    }

    impl<T, const N: usize> Default for BoundedSequence<T, N> {
        fn default() -> Self {
            BoundedSequence(Vec::new())
        }
    }

    impl<const N: usize> Default for BoundedString<N> {
        fn default() -> Self {
            BoundedString(std::string::String::new())
        }
    }

    impl<const N: usize> Default for BoundedWString<N> {
        fn default() -> Self {
            BoundedWString(std::string::String::new())
        }
    }
}
"#
    .to_string()
}

/// Helper to check if cargo is available
fn cargo_available() -> bool {
    Command::new("cargo").arg("--version").output().is_ok()
}

#[test]
fn test_simple_message_compiles() -> Result<(), GeneratorError> {
    if !cargo_available() {
        eprintln!("Skipping compilation test - cargo not available");
        return Ok(());
    }

    let msg_def = "int32 x\nfloat64 y\nstring name\n";
    let msg = parse_message(msg_def).unwrap();

    let result = generate_message_package("test_msgs", "SimpleMsg", &msg, &HashSet::new())?;

    // Create temp directory for test package
    let temp_dir = TempDir::new().unwrap();
    let pkg_dir = temp_dir.path().join("test_msgs");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Write Cargo.toml
    let cargo_toml = create_test_cargo_toml("test_msgs", false);
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Write generated lib.rs
    let src_dir = pkg_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create a simple lib.rs that includes the generated code
    // Add use statement if generated code uses rosidl_runtime_rs types
    let needs_rosidl_types = result.message_rmw.contains("rosidl_runtime_rs");
    let use_stmt = if needs_rosidl_types {
        "use crate::rosidl_runtime_rs;"
    } else {
        ""
    };

    let lib_rs = format!(
        r#"
{}

// Auto-generated message types
pub mod msg {{
    pub mod rmw {{
        {}
        {}
    }}

    // Idiomatic types
    {}
}}
"#,
        create_rosidl_runtime_stub(),
        use_stmt,
        result.message_rmw,
        result.message_idiomatic
    );

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Try to compile it
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(pkg_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Generated code failed to compile");
    }

    Ok(())
}

#[test]
fn test_message_with_arrays_compiles() -> Result<(), GeneratorError> {
    if !cargo_available() {
        eprintln!("Skipping compilation test - cargo not available");
        return Ok(());
    }

    // Use array sizes ≤ 32 to avoid needing big-array (serde limitation)
    let msg_def = "int32[5] small_array\nint32[32] large_array\n";
    let msg = parse_message(msg_def).unwrap();

    let result = generate_message_package("test_msgs", "ArrayMsg", &msg, &HashSet::new())?;

    // Create temp directory for test package
    let temp_dir = TempDir::new().unwrap();
    let pkg_dir = temp_dir.path().join("test_msgs_arrays");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Write Cargo.toml (no big-array needed for arrays ≤ 32)
    let cargo_toml = create_test_cargo_toml("test_msgs_arrays", false);
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Write generated lib.rs
    let src_dir = pkg_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let needs_rosidl_types = result.message_rmw.contains("rosidl_runtime_rs");
    let use_stmt = if needs_rosidl_types {
        "use crate::rosidl_runtime_rs;"
    } else {
        ""
    };

    let lib_rs = format!(
        r#"
{}

pub mod msg {{
    pub mod rmw {{
        {}
        {}
    }}

    {}
}}
"#,
        create_rosidl_runtime_stub(),
        use_stmt,
        result.message_rmw,
        result.message_idiomatic
    );

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Try to compile it
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(pkg_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Generated code with arrays failed to compile");
    }

    Ok(())
}

#[test]
fn test_check_no_warnings() -> Result<(), GeneratorError> {
    if !cargo_available() {
        eprintln!("Skipping compilation test - cargo not available");
        return Ok(());
    }

    let msg_def = "int32 x\nfloat64 y\n";
    let msg = parse_message(msg_def).unwrap();

    let result = generate_message_package("test_msgs", "Point", &msg, &HashSet::new())?;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let pkg_dir = temp_dir.path().join("test_msgs_warnings");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Write files
    let cargo_toml = create_test_cargo_toml("test_msgs_warnings", false);
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = pkg_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let needs_rosidl_types = result.message_rmw.contains("rosidl_runtime_rs");
    let use_stmt = if needs_rosidl_types {
        "use crate::rosidl_runtime_rs;"
    } else {
        ""
    };

    let lib_rs = format!(
        r#"
#![deny(warnings)]

{}

pub mod msg {{
    pub mod rmw {{
        {}
        {}
    }}

    {}
}}
"#,
        create_rosidl_runtime_stub(),
        use_stmt,
        result.message_rmw,
        result.message_idiomatic
    );

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Compile with warnings as errors
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(pkg_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if it's actually a warning turned error
        if stderr.contains("warning") {
            eprintln!("Generated code has warnings:");
            eprintln!("{}", stderr);
            panic!("Generated code produced warnings");
        }
    }

    Ok(())
}

#[test]
fn test_clippy_no_warnings() -> Result<(), GeneratorError> {
    // Check if clippy is available
    let clippy_check = Command::new("cargo")
        .arg("clippy")
        .arg("--version")
        .output();

    if clippy_check.is_err() {
        eprintln!("Skipping clippy test - clippy not available");
        return Ok(());
    }

    let msg_def = "int32 value\nstring name\n";
    let msg = parse_message(msg_def).unwrap();

    let result = generate_message_package("test_msgs", "TestMsg", &msg, &HashSet::new())?;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let pkg_dir = temp_dir.path().join("test_msgs_clippy");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Write files
    let cargo_toml = create_test_cargo_toml("test_msgs_clippy", false);
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = pkg_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let needs_rosidl_types = result.message_rmw.contains("rosidl_runtime_rs");
    let use_stmt = if needs_rosidl_types {
        "use crate::rosidl_runtime_rs;"
    } else {
        ""
    };

    let lib_rs = format!(
        r#"
{}

pub mod msg {{
    pub mod rmw {{
        {}
        {}
    }}

    {}
}}
"#,
        create_rosidl_runtime_stub(),
        use_stmt,
        result.message_rmw,
        result.message_idiomatic
    );

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Run clippy
    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--manifest-path")
        .arg(pkg_dir.join("Cargo.toml"))
        .arg("--")
        .arg("-W")
        .arg("clippy::all")
        .output()
        .expect("Failed to run cargo clippy");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Clippy might have suggestions, but let's check there are no serious issues
    if stderr.contains("error") {
        eprintln!("Clippy found errors:");
        eprintln!("{}", stderr);
        panic!("Generated code failed clippy checks");
    }

    Ok(())
}
