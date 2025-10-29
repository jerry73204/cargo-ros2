# cargo-ros2: Technical Design Document

**Version**: 1.0
**Date**: 2025-01-29
**Status**: Design Phase

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Design Goals](#design-goals)
3. [Architecture Overview](#architecture-overview)
4. [Core Components](#core-components)
5. [Data Flow](#data-flow)
6. [API Design](#api-design)
7. [Cache Management](#cache-management)
8. [Edge Cases & Error Handling](#edge-cases--error-handling)
9. [Performance Considerations](#performance-considerations)
10. [Security Considerations](#security-considerations)
11. [Future Extensions](#future-extensions)

---

## 1. Problem Statement

### 1.1 The Circular Dependency Problem

ROS 2 Rust bindings face a fundamental chicken-and-egg problem:

```
1. User writes Cargo.toml:
   [dependencies]
   vision_msgs = "*"

2. Cargo resolves dependencies:
   - Queries crates.io for vision_msgs
   - Finds yanked placeholder version

3. .cargo/config.toml patch attempts redirect:
   [patch.crates-io]
   vision_msgs = { path = "install/vision_msgs/.../rust" }

4. ERROR: Path doesn't exist yet!
   - Rust bindings need to be generated first
   - But generation happens during build
   - Cargo dependency resolution happens BEFORE build
   - Circular dependency!
```

### 1.2 Why Current Solutions Fail

**ros2_rust (official)**:
- Requires colcon workspace
- 3-stage build: ros2_rust → interfaces → packages
- Doesn't work with system-installed ROS packages
- Complex setup for simple projects

**r2r (alternative)**:
- Generates in build.rs (avoids circular dep)
- Regenerates ALL bindings every build (slow)
- Non-standard API (no Cargo.toml deps)
- Harder to debug (generated code in OUT_DIR)

### 1.3 Our Solution in One Sentence

**Generate bindings to `target/ros2_bindings/` BEFORE Cargo's dependency resolution, then patch Cargo to use them.**

---

## 2. Design Goals

### 2.1 Primary Goals

1. **Break Circular Dependency**: Bindings exist before Cargo resolves deps
2. **System Package Support**: Discover ROS packages via `ament_index`
3. **Standard Cargo Experience**: Normal Cargo.toml, transparent patches
4. **Project Isolation**: All artifacts in `target/`, no global state
5. **Zero Configuration**: User just runs `cargo ros2 build`

### 2.2 Secondary Goals

6. **Incremental Builds**: Smart caching (checksum-based)
7. **colcon Integration**: Drop-in replacement for `cargo build`
8. **Multi-Distro Support**: Detect ROS_DISTRO, handle version differences
9. **IDE Compatibility**: Works with rust-analyzer
10. **Debuggability**: Clear error messages, easy to inspect generated code

### 2.3 Non-Goals (for MVP)

- GUI tools (CLI only)
- Custom message format support (ROS IDL only)
- Cross-compilation
- Windows support (Linux/macOS first)

---

## 3. Architecture Overview

### 3.1 High-Level Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     User invokes:                           │
│                  $ cargo ros2 build                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Phase 1: Pre-Build Analysis                                │
│  ─────────────────────────                                  │
│  1. Parse Cargo.toml                                        │
│  2. Extract ROS dependencies (vision_msgs, sensor_msgs, ...) │
│  3. Check cache (.ros2_bindgen_cache)                       │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Phase 2: Binding Generation                                │
│  ────────────────────────                                   │
│  For each ROS package:                                      │
│    1. Discover via ament_index                              │
│    2. Parse .msg/.srv/.action files                         │
│    3. Generate Rust FFI code                                │
│    4. Write to target/ros2_bindings/<pkg>/                  │
│    5. Update cache                                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Phase 3: Patch Setup                                       │
│  ──────────────────                                         │
│  1. Create/update .cargo/config.toml                        │
│  2. Add [patch.crates-io] entries                           │
│  3. Point to target/ros2_bindings/<pkg>/                    │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Phase 4: Build                                             │
│  ────────────                                               │
│  1. exec() into cargo build                                 │
│  2. Cargo resolves deps → patches redirect to local paths   │
│  3. Compiles successfully!                                  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Component Architecture

```
cargo-ros2/
├── cargo-ros2-cli/          # Cargo subcommand binary
│   └── src/
│       └── main.rs          # Entry point, CLI parsing
├── cargo-ros2-core/         # Core library
│   └── src/
│       ├── lib.rs
│       ├── discovery.rs     # ament_index integration
│       ├── generator.rs     # Binding generation
│       ├── cache.rs         # Cache management
│       ├── patcher.rs       # .cargo/config.toml management
│       └── parser.rs        # ROS IDL parsing
├── cargo-ros2-codegen/      # Code generation utilities
│   └── src/
│       ├── lib.rs
│       ├── msg.rs           # Message type generation
│       ├── srv.rs           # Service type generation
│       ├── action.rs        # Action type generation
│       └── templates/       # Code templates
└── tests/
    ├── integration/         # End-to-end tests
    └── fixtures/            # Mock ROS packages
```

---

## 4. Core Components

### 4.1 Discovery Module (`discovery.rs`)

**Purpose**: Locate ROS packages using `ament_index`

```rust
pub struct PackageDiscovery {
    ament_prefix_path: Vec<PathBuf>,
}

impl PackageDiscovery {
    pub fn new() -> Result<Self>;

    /// Find a ROS package by name
    pub fn find_package(&self, name: &str) -> Result<PackageInfo>;

    /// Get all available message packages
    pub fn list_message_packages(&self) -> Result<Vec<String>>;
}

pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub msg_dir: Option<PathBuf>,
    pub srv_dir: Option<PathBuf>,
    pub action_dir: Option<PathBuf>,
    pub dependencies: Vec<String>,
}
```

**Implementation Details**:
- Parse `AMENT_PREFIX_PATH` environment variable
- Search `<prefix>/share/<package>/` for package.xml
- Parse package.xml for dependencies
- Validate ROS_DISTRO compatibility

### 4.2 Generator Module (`generator.rs`)

**Purpose**: Generate Rust crate for each ROS package

```rust
pub struct BindingGenerator {
    output_dir: PathBuf,
    codegen: CodeGenerator,
}

impl BindingGenerator {
    pub fn new(output_dir: PathBuf) -> Self;

    /// Generate bindings for a single ROS package
    pub fn generate(&self, pkg_info: &PackageInfo) -> Result<()>;

    /// Check if regeneration needed (cache miss or stale)
    pub fn needs_regeneration(&self, pkg_name: &str) -> Result<bool>;
}

struct GeneratedPackage {
    cargo_toml: String,      // Package manifest
    lib_rs: String,          // Public API (pub mod msg/srv/action)
    msg_mod_rs: String,      // Message implementations
    srv_mod_rs: String,      // Service implementations
    action_mod_rs: String,   // Action implementations
    build_rs: String,        // FFI linking
}
```

**Generated Crate Structure**:
```
target/ros2_bindings/vision_msgs/
├── Cargo.toml              # Generated manifest
├── build.rs                # Links C typesupport libs
└── src/
    ├── lib.rs              # pub mod msg; pub mod srv;
    └── msg/
        ├── mod.rs          # pub mod detection3d; ...
        ├── detection3d.rs  # struct Detection3D { ... }
        └── ...
```

**Generated Cargo.toml**:
```toml
[package]
name = "vision_msgs"
version = "4.1.0"  # Matches ROS package version
edition = "2021"

[dependencies]
# Transitive ROS deps
std_msgs = "*"
geometry_msgs = "*"
sensor_msgs = "*"

[build-dependencies]
# None needed (linking handled in build.rs)
```

**Generated build.rs**:
```rust
fn main() {
    // Link to ROS C typesupport library
    let ros_distro = std::env::var("ROS_DISTRO").unwrap_or_else(|_| "humble".into());
    let lib_path = format!("/opt/ros/{}/lib", ros_distro);

    println!("cargo:rustc-link-search={}", lib_path);
    println!("cargo:rustc-link-lib=vision_msgs__rosidl_typesupport_c");

    // Re-run if package.xml changes
    println!("cargo:rerun-if-changed=/opt/ros/{}/share/vision_msgs/package.xml", ros_distro);
}
```

### 4.3 Cache Module (`cache.rs`)

**Purpose**: Track generated bindings to avoid regeneration

```rust
pub struct BindingCache {
    cache_file: PathBuf,  // .ros2_bindgen_cache
    entries: HashMap<String, CacheEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub package_name: String,
    pub version: String,
    pub checksum: String,          // SHA256 of .msg/.srv/.action files
    pub generated_at: SystemTime,
    pub ros_distro: String,
    pub generator_version: String, // cargo-ros2 version
}

impl BindingCache {
    pub fn load(project_root: &Path) -> Result<Self>;
    pub fn save(&self) -> Result<()>;

    /// Check if cached binding is still valid
    pub fn is_fresh(&self, pkg_info: &PackageInfo) -> bool;

    /// Update cache after generation
    pub fn update(&mut self, pkg_name: &str, checksum: &str);
}
```

**Cache Format** (`.ros2_bindgen_cache`):
```json
{
  "version": "1.0",
  "ros_distro": "humble",
  "generator_version": "0.1.0",
  "packages": {
    "vision_msgs": {
      "version": "4.1.0",
      "checksum": "abc123def456...",
      "generated_at": "2025-01-29T12:34:56Z"
    },
    "sensor_msgs": {
      "version": "4.2.3",
      "checksum": "def789ghi012...",
      "generated_at": "2025-01-29T12:34:57Z"
    }
  }
}
```

**Invalidation Triggers**:
- Message definition files changed (different checksum)
- ROS_DISTRO changed
- Package version changed
- cargo-ros2 version changed (generator improvements)

### 4.4 Patcher Module (`patcher.rs`)

**Purpose**: Manage `.cargo/config.toml` patches

```rust
pub struct CargoPatcher {
    config_path: PathBuf,  // .cargo/config.toml
}

impl CargoPatcher {
    pub fn new(project_root: &Path) -> Self;

    /// Add patches for ROS packages
    pub fn add_patches(&self, packages: &[String], bindings_dir: &Path) -> Result<()>;

    /// Remove stale patches (packages no longer used)
    pub fn clean_stale_patches(&self, active_packages: &[String]) -> Result<()>;

    /// Validate existing patches
    pub fn validate(&self) -> Result<Vec<String>>;  // Returns list of issues
}
```

**Generated .cargo/config.toml**:
```toml
# AUTO-GENERATED by cargo-ros2
# DO NOT EDIT - changes will be overwritten
# Last updated: 2025-01-29 12:34:56

[patch.crates-io]
vision_msgs = { path = "target/ros2_bindings/vision_msgs" }
sensor_msgs = { path = "target/ros2_bindings/sensor_msgs" }
geometry_msgs = { path = "target/ros2_bindings/geometry_msgs" }
std_msgs = { path = "target/ros2_bindings/std_msgs" }
```

**Preservation Strategy**:
- Parse existing config.toml
- Preserve non-ros2 sections
- Only update `[patch.crates-io]` entries for ROS packages
- Add comments for traceability

---

## 5. Data Flow

### 5.1 Dependency Extraction

```rust
fn extract_ros_dependencies(manifest: &CargoToml) -> Vec<String> {
    let mut deps = Vec::new();

    // Check [dependencies]
    for (name, _spec) in &manifest.dependencies {
        if is_ros_package(name) {
            deps.push(name.clone());
        }
    }

    // Check [dev-dependencies]
    for (name, _spec) in &manifest.dev_dependencies {
        if is_ros_package(name) {
            deps.push(name.clone());
        }
    }

    deps
}

fn is_ros_package(name: &str) -> bool {
    // Heuristic: Check if package exists in ament_index
    ament_index::get_package_share_directory(name).is_ok()
}
```

### 5.2 Transitive Dependency Resolution

```rust
fn resolve_transitive_deps(
    pkg_name: &str,
    discovery: &PackageDiscovery,
) -> Result<Vec<String>> {
    let mut resolved = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(pkg_name.to_string());

    while let Some(current) = queue.pop_front() {
        if resolved.contains(&current) {
            continue;
        }

        let pkg_info = discovery.find_package(&current)?;
        resolved.insert(current.clone());

        // Add message package dependencies
        for dep in &pkg_info.dependencies {
            if is_message_package(dep) {
                queue.push_back(dep.clone());
            }
        }
    }

    Ok(resolved.into_iter().collect())
}
```

### 5.3 Binding Generation Pipeline

```
.msg file → Parser → AST → Code Generator → Rust file

Example:
detection3d.msg:
  Header header
  BoundingBox3D bbox
  float32 score

↓ Parse

AST:
  Message {
    name: "Detection3D",
    fields: [
      Field { name: "header", type: "Header", pkg: "std_msgs" },
      Field { name: "bbox", type: "BoundingBox3D", pkg: "vision_msgs" },
      Field { name: "score", type: "f32", primitive: true },
    ]
  }

↓ Generate

detection3d.rs:
  #[repr(C)]
  pub struct Detection3D {
      pub header: std_msgs::msg::Header,
      pub bbox: BoundingBox3D,
      pub score: f32,
  }

  impl Default for Detection3D { ... }
  impl Detection3D {
      pub fn to_native(&self) -> *mut rcl_detection3d_t { ... }
      pub fn from_native(ptr: *const rcl_detection3d_t) -> Self { ... }
  }
```

---

## 6. API Design

### 6.1 CLI Interface

```bash
# Primary command
cargo ros2 build [OPTIONS]

Options:
  --release              Build in release mode
  --target <TRIPLE>      Build for target triple
  --package <SPEC>       Package to build
  --workspace            Build all workspace members
  --verbose              Verbose output
  --no-cache             Ignore cache, regenerate all
  --bindings-only        Generate bindings only, don't build

# Auxiliary commands
cargo ros2 check           # Fast check (reuses bindings)
cargo ros2 clean           # Clean target/ including bindings
cargo ros2 cache --list    # List cached bindings
cargo ros2 cache --rebuild # Force regeneration
cargo ros2 info <PACKAGE>  # Show ROS package info
```

### 6.2 Library API (for colcon integration)

```rust
// Public API for colcon-ros-cargo plugin
pub struct Ros2Builder {
    project_root: PathBuf,
    config: BuildConfig,
}

pub struct BuildConfig {
    pub release: bool,
    pub target: Option<String>,
    pub no_cache: bool,
    pub verbose: bool,
}

impl Ros2Builder {
    pub fn new(project_root: PathBuf) -> Result<Self>;
    pub fn with_config(config: BuildConfig) -> Self;

    /// Pre-build: Generate bindings
    pub fn prepare(&self) -> Result<()>;

    /// Build: Invoke cargo
    pub fn build(&self) -> Result<()>;

    /// Complete workflow
    pub fn run(&self) -> Result<()> {
        self.prepare()?;
        self.build()
    }
}
```

---

## 7. Cache Management

### 7.1 Checksum Calculation

```rust
fn calculate_package_checksum(pkg_info: &PackageInfo) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash all .msg files
    if let Some(msg_dir) = &pkg_info.msg_dir {
        for entry in fs::read_dir(msg_dir)? {
            let path = entry?.path();
            if path.extension() == Some("msg") {
                let content = fs::read(&path)?;
                hasher.update(&content);
            }
        }
    }

    // Hash all .srv files
    // ... similar for srv_dir

    // Hash all .action files
    // ... similar for action_dir

    // Hash package version
    hasher.update(pkg_info.version.as_bytes());

    Ok(format!("{:x}", hasher.finalize()))
}
```

### 7.2 Cache Invalidation Strategy

**Invalidate when**:
1. Checksum mismatch (source files changed)
2. ROS_DISTRO changed
3. Package version changed
4. Generator version changed (cargo-ros2 updated)
5. Manual `--no-cache` flag

**Retain when**:
- Cargo.toml dependencies added/removed (only affects which packages are generated)
- User code changes (bindings unchanged)
- Build artifacts cleaned (`cargo clean` preserves bindings by default)

### 7.3 Cache Storage Location

```
# Option A: Project-local (chosen)
.ros2_bindgen_cache

# Option B: Global cache (rejected - not project-isolated)
~/.cache/cargo-ros2/

# Reasoning:
# - Project-local ensures isolation
# - Different projects can use different ROS versions
# - Survives git operations (in .gitignore)
# - Cleaned with `cargo ros2 clean`
```

---

## 8. Edge Cases & Error Handling

### 8.1 Missing ROS Installation

**Scenario**: User hasn't sourced ROS setup.bash

```
Error: ROS 2 not found
  ├─ AMENT_PREFIX_PATH is not set
  ├─ Please source your ROS 2 installation:
  │    $ source /opt/ros/humble/setup.bash
  └─ Or set AMENT_PREFIX_PATH manually
```

**Detection**:
```rust
fn check_ros_environment() -> Result<()> {
    if std::env::var("AMENT_PREFIX_PATH").is_err() {
        return Err(Error::RosNotFound);
    }
    Ok(())
}
```

### 8.2 Package Not Found

**Scenario**: User depends on non-existent package

```
Error: ROS package 'my_custom_msgs' not found
  ├─ Searched in AMENT_PREFIX_PATH:
  │    - /opt/ros/humble
  │    - /home/user/ros2_ws/install
  ├─ Did you forget to source the workspace?
  └─ Or install the package?
```

### 8.3 Conflicting ROS Distros

**Scenario**: Workspace uses Humble, system has Iron

```
Warning: Multiple ROS distributions detected
  ├─ Workspace: humble (from .ros2_bindgen_cache)
  ├─ Current:   iron (from ROS_DISTRO)
  ├─ This may cause binary incompatibilities
  └─ Recommendation: Source the same distro or clean cache
```

### 8.4 Stale Patches

**Scenario**: .cargo/config.toml has patches but bindings were deleted

```
Error: Stale patch detected
  ├─ Package: vision_msgs
  ├─ Patch points to: target/ros2_bindings/vision_msgs
  ├─ But path doesn't exist
  └─ Run: cargo ros2 cache --rebuild
```

### 8.5 Circular Message Dependencies

**Scenario**: msg_a.msg references msg_b.msg, msg_b.msg references msg_a.msg

```
Error: Circular dependency detected
  ├─ Cycle: my_pkg::MsgA → my_pkg::MsgB → my_pkg::MsgA
  └─ ROS message definitions should form a DAG
```

**Handling**: Detect cycles during dependency resolution, report error with cycle path.

---

## 9. Performance Considerations

### 9.1 Binding Generation Cost

**Benchmarks** (estimated, to be validated):
- Parse 1 .msg file: ~100μs
- Generate 1 Rust struct: ~500μs
- Compile 1 binding crate: ~2s (debug), ~5s (release)

**Total for typical project**:
- 20 ROS packages × 2s = 40s one-time cost
- Subsequent builds: 0s (cached)

### 9.2 Optimization Strategies

1. **Parallel Generation**: Generate multiple packages concurrently
   ```rust
   use rayon::prelude::*;

   packages.par_iter().for_each(|pkg| {
       generator.generate(pkg).unwrap();
   });
   ```

2. **Incremental Compilation**: Pre-compile bindings to .rlib files
   ```bash
   cd target/ros2_bindings/vision_msgs
   cargo build --release  # Creates .rlib
   ```

3. **Lazy Loading**: Only generate packages actually imported in user code
   (Future optimization)

### 9.3 Cache Hit Rate

**Target**: >95% cache hits in typical development workflow

**Analysis**:
- Message definitions change rarely (stable across months)
- Generator version updates infrequent (quarterly releases)
- ROS distro changes rare (annual upgrades)

**Expected performance**: Cold build: 60s, Hot build: 5s (10x improvement)

---

## 10. Security Considerations

### 10.1 Path Injection

**Risk**: Malicious .cargo/config.toml with arbitrary paths

**Mitigation**:
- Validate all paths are within `target/ros2_bindings/`
- Reject absolute paths outside project
- Sanitize package names (alphanumeric + underscore only)

```rust
fn validate_binding_path(path: &Path, project_root: &Path) -> Result<()> {
    let canonical = path.canonicalize()?;
    let bindings_dir = project_root.join("target/ros2_bindings");

    if !canonical.starts_with(&bindings_dir) {
        return Err(Error::InvalidPath);
    }

    Ok(())
}
```

### 10.2 Code Injection

**Risk**: Malicious .msg file with shell commands

**Mitigation**:
- Parse .msg files with strict grammar (no shell execution)
- Use safe code generation (no format! with user input)
- Validate all identifiers match Rust syntax

```rust
fn validate_identifier(name: &str) -> Result<()> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidIdentifier);
    }
    Ok(())
}
```

### 10.3 Supply Chain

**Risk**: Compromised ROS packages

**Mitigation**:
- Checksum verification against known-good values (future)
- Signature verification (if ROS adds signing)
- Audit logging of generated code

---

## 11. Future Extensions

### 11.1 IDE Integration

**Goal**: rust-analyzer understands ROS packages

**Challenge**: Generated code in `target/` not visible to IDEs

**Solution**:
```rust
// Generate rust-project.json for rust-analyzer
{
  "sysroot_src": "...",
  "crates": [
    {
      "root_module": "target/ros2_bindings/vision_msgs/src/lib.rs",
      "edition": "2021",
      "deps": [...]
    }
  ]
}
```

### 11.2 Custom Message Formats

**Goal**: Support non-ROS message formats (Protobuf, FlatBuffers)

**Design**:
```rust
trait MessageFormat {
    fn parse(&self, file: &Path) -> Result<Ast>;
    fn generate(&self, ast: &Ast) -> Result<String>;
}

struct RosIdlFormat;
struct ProtobufFormat;
struct FlatBuffersFormat;
```

### 11.3 Cross-Compilation

**Goal**: Build for embedded targets (ARM, RISC-V)

**Challenges**:
- C typesupport libraries need cross-compilation
- ament_index may not work on target

**Solution**:
- Bundle pre-compiled typesupport for common targets
- Allow specifying alternate AMENT_PREFIX_PATH for target

### 11.4 Hot Reload

**Goal**: Regenerate bindings on .msg file changes without restart

**Design**:
- Watch mode: `cargo ros2 watch`
- File system watcher on `ament_index` paths
- Incremental regeneration on change
- Trigger `cargo check` automatically

---

## Appendix A: Comparison with Alternatives

| Feature | cargo-ros2 | ros2_rust | r2r |
|---------|------------|-----------|-----|
| **Circular Dep** | ✅ Solved | ❌ Requires workspace | ✅ Solved |
| **System Packages** | ✅ ament_index | ❌ Needs local build | ✅ ament_index |
| **Build Speed** | ✅ Cached | ✅ Pre-compiled | ❌ Regenerates all |
| **Standard Cargo** | ✅ Normal deps | ⚠️ Patches | ❌ No Cargo.toml deps |
| **IDE Support** | ✅ rust-analyzer | ✅ rust-analyzer | ⚠️ Generated in OUT_DIR |
| **colcon Integration** | ✅ Drop-in | ✅ Native | ⚠️ Manual |
| **Maintenance** | 🆕 New | ✅ Official | ✅ Active |

---

## Appendix B: Message Format Examples

### Input: .msg file
```
# vision_msgs/Detection3D.msg
std_msgs/Header header
BoundingBox3D bbox
float32 score
string class_id
```

### Output: Rust struct
```rust
// target/ros2_bindings/vision_msgs/src/msg/detection3d.rs
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Detection3D {
    pub header: std_msgs::msg::Header,
    pub bbox: super::BoundingBox3D,
    pub score: f32,
    pub class_id: String,
}

impl Default for Detection3D {
    fn default() -> Self {
        Self {
            header: Default::default(),
            bbox: Default::default(),
            score: 0.0,
            class_id: String::new(),
        }
    }
}

// FFI bindings to C typesupport
extern "C" {
    fn vision_msgs__msg__Detection3D__init(msg: *mut Detection3D) -> bool;
    fn vision_msgs__msg__Detection3D__fini(msg: *mut Detection3D);
    // ... more FFI functions
}
```

---

**End of Design Document**
