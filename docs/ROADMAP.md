# cargo-ros2: Implementation Roadmap

**Version**: 1.0
**Created**: 2025-01-29
**Status**: Phase 0 (Design)

---

## Overview

This roadmap outlines the phased implementation of cargo-ros2, a project-centric ROS 2 Rust binding system. Each phase builds incrementally toward a production-ready tool.

**Target Timeline**: ~12-16 weeks for Phases 1-3

---

## Phase 0: Design & Planning ✅

**Duration**: 1 week (Complete: 2025-01-29)

**Deliverables**:
- [x] CLAUDE.md (Project instructions)
- [x] docs/DESIGN.md (Technical design)
- [x] docs/ROADMAP.md (This document)
- [x] Architecture diagrams
- [x] API specification

**Outcomes**:
- Clear problem definition
- Validated approach (project-local bindings)
- Implementation plan agreed

---

## Phase 1: MVP (Messages Only)

**Duration**: 3-4 weeks
**Goal**: Demonstrate core concept with message types only

### Milestones

#### 1.1: Project Setup (Week 1)
- [ ] Create repository structure
  - [ ] `cargo-ros2-cli/` (binary crate)
  - [ ] `cargo-ros2-core/` (library crate)
  - [ ] `cargo-ros2-codegen/` (code generation)
- [ ] Set up CI/CD (GitHub Actions)
  - [ ] Rust fmt/clippy checks
  - [ ] Unit tests
  - [ ] Integration tests
- [ ] Configure crate metadata
  - [ ] Cargo.toml with metadata
  - [ ] README.md
  - [ ] LICENSE (MIT + Apache-2.0)

#### 1.2: Discovery Module (Week 1-2)
- [ ] Implement `PackageDiscovery`
  - [ ] Parse `AMENT_PREFIX_PATH`
  - [ ] Locate packages via `share/<pkg>/package.xml`
  - [ ] Parse `package.xml` for metadata
  - [ ] Extract dependencies
- [ ] Unit tests with mock filesystem
- [ ] Integration test with real ROS installation

**Acceptance Criteria**:
```rust
let discovery = PackageDiscovery::new()?;
let info = discovery.find_package("std_msgs")?;
assert_eq!(info.name, "std_msgs");
assert!(info.msg_dir.is_some());
```

#### 1.3: Message Parser (Week 2)
- [ ] Implement .msg file parser
  - [ ] Lexer (tokenize .msg syntax)
  - [ ] Parser (build AST)
  - [ ] Validate message definitions
- [ ] Support primitive types (bool, int, float, string)
- [ ] Support nested messages (pkg/MsgType)
- [ ] Support arrays (fixed + dynamic)
- [ ] Unit tests for all syntax

**Acceptance Criteria**:
```rust
let ast = parse_msg_file("std_msgs/Header.msg")?;
assert_eq!(ast.name, "Header");
assert_eq!(ast.fields.len(), 3); // stamp, frame_id, ...
```

#### 1.4: Code Generator (Week 2-3)
- [ ] Implement Rust code generator
  - [ ] Generate struct definitions
  - [ ] Generate `Default` impl
  - [ ] Generate FFI extern declarations
  - [ ] Generate build.rs for linking
- [ ] Use templates for consistency
- [ ] Unit tests for generated code

**Acceptance Criteria**:
```rust
let rust_code = generate_message(&ast)?;
// Compiles successfully
// Passes clippy
// Matches expected output
```

#### 1.5: Binding Generator (Week 3)
- [ ] Implement `BindingGenerator`
  - [ ] Create target/ros2_bindings/<pkg>/
  - [ ] Generate Cargo.toml
  - [ ] Generate src/lib.rs + mod structure
  - [ ] Generate build.rs
- [ ] Handle transitive dependencies
- [ ] Integration test: generate std_msgs

**Acceptance Criteria**:
```bash
cargo ros2 build
# Generates target/ros2_bindings/std_msgs/
# Contains valid Cargo project
# Compiles successfully
```

#### 1.6: CLI & Integration (Week 3-4)
- [ ] Implement `cargo-ros2` CLI
  - [ ] `cargo ros2 build` subcommand
  - [ ] Parse Cargo.toml for dependencies
  - [ ] Call binding generator
  - [ ] Setup .cargo/config.toml patches
  - [ ] exec() into cargo build
- [ ] Error handling & user feedback
- [ ] End-to-end test with sample project

**Acceptance Criteria**:
```bash
# User project
cat Cargo.toml
  [dependencies]
  std_msgs = "*"

cargo ros2 build
# Success!
# Can use std_msgs::msg::Header in code
```

### Phase 1 Deliverables

- [x] Core library (discovery, parser, generator)
- [x] CLI tool (cargo ros2 build)
- [x] Documentation (usage guide)
- [x] Tests (unit + integration)
- [x] Example project demonstrating usage

### Phase 1 Success Metrics

- ✅ Generates bindings for 10+ common message packages
- ✅ End-to-end build succeeds without errors
- ✅ Generated code passes clippy
- ✅ Build time < 60s for cold build
- ✅ Cache hit reduces build to < 5s

---

## Phase 2: Services, Actions, Cache, Ament Installation

**Duration**: 4-5 weeks
**Goal**: Complete feature set for production use + ament installation (absorbing cargo-ament-build)

**📖 See `docs/UNIFIED_ARCHITECTURE.md` for the complete unified architecture design.**

### Milestones

#### 2.1: Service Support (Week 1)
- [ ] Extend parser for .srv files
  - [ ] Request/Response separation
  - [ ] Constants
- [ ] Code generator for services
  - [ ] Generate request/response structs
  - [ ] Generate service traits
  - [ ] FFI bindings for services
- [ ] Integration tests

**Example Generated Code**:
```rust
pub mod add_two_ints {
    pub struct Request {
        pub a: i64,
        pub b: i64,
    }

    pub struct Response {
        pub sum: i64,
    }
}
```

#### 2.2: Action Support (Week 1-2)
- [ ] Extend parser for .action files
  - [ ] Goal/Result/Feedback sections
- [ ] Code generator for actions
  - [ ] Generate goal/result/feedback structs
  - [ ] Generate action traits
  - [ ] FFI bindings for actions
- [ ] Integration tests

#### 2.3: Cache System (Week 2)
- [ ] Implement `BindingCache`
  - [ ] JSON serialization (serde)
  - [ ] Checksum calculation (SHA256)
  - [ ] Freshness validation
  - [ ] Load/save operations
- [ ] Integrate with generator
  - [ ] Skip generation if cache hit
  - [ ] Update cache on generation
- [ ] Unit tests

**Acceptance Criteria**:
```bash
cargo ros2 build              # 60s (cold)
cargo ros2 build              # 2s (cache hit)
# Edit .msg file
cargo ros2 build              # 15s (partial regeneration)
```

#### 2.4: Enhanced CLI (Week 2-3)
- [ ] Add `cargo ros2 cache` subcommands
  - [ ] `--list`: Show cached packages
  - [ ] `--rebuild`: Force regeneration
  - [ ] `--clean`: Remove cache
- [ ] Add `cargo ros2 info <pkg>` command
  - [ ] Show package metadata
  - [ ] Show dependencies
  - [ ] Show cached status
- [ ] Improve error messages
  - [ ] Structured diagnostics
  - [ ] Helpful suggestions

#### 2.5: Performance Optimization (Week 3)
- [ ] Parallel binding generation (rayon)
- [ ] Pre-compilation of bindings to .rlib
- [ ] Optimize checksum calculation
- [ ] Profiling & benchmarking

**Target**: Cold build < 30s, Hot build < 2s

#### 2.6: Ament Installation (Week 3-4) **[NEW - UNIFIED ARCHITECTURE]**

**Background**: Absorbing cargo-ament-build functionality into cargo-ros2 for a unified tool.

- [ ] Extract installation code from cargo-ament-build (in `tmp/cargo-ament-build/`)
  - [ ] `create_package_marker()` - ament_index marker creation
  - [ ] `install_package()` - Source code installation
  - [ ] `install_binaries()` - Binary/library installation
  - [ ] `install_files_from_metadata()` - Metadata-based file installation
  - [ ] Pure library detection logic (check vs build)
- [ ] Implement new modules in cargo-ros2-core
  - [ ] `src/installer.rs` - Main installation orchestration
  - [ ] `src/marker.rs` - Marker creation
  - [ ] `src/source_install.rs` - Source installation
  - [ ] `src/binary_install.rs` - Binary installation
  - [ ] `src/metadata_install.rs` - Metadata installation
- [ ] Implement `AmentBuilder` struct
  - [ ] Orchestrate: generate → build → install
  - [ ] Handle build vs check decision
  - [ ] Parse features, profiles, architectures
- [ ] Implement `cargo ros2 ament-build` command
  - [ ] Parse `--install-base`, `--build-base` args
  - [ ] Forward cargo args after `--`
  - [ ] Invoke AmentBuilder
- [ ] Add dependencies to Cargo.toml
  - [ ] `cargo-manifest = "0.17"` (Cargo.toml parsing)
  - [ ] `anyhow = "1.0"` (error handling)
- [ ] Integration tests
  - [ ] Test with real ROS 2 package
  - [ ] Verify ament layout structure
  - [ ] Compare with cargo-ament-build output
  - [ ] Test metadata installation

**Acceptance Criteria**:
```bash
cargo ros2 ament-build --install-base install/my_pkg -- --release
# → Generates bindings (if needed)
# → Builds package
# → Installs to install/my_pkg/ with ament layout:
#    - lib/my_pkg/ (binaries)
#    - share/my_pkg/rust/ (source)
#    - share/my_pkg/ (package.xml)
#    - share/ament_index/resource_index/ (markers)
# → Output identical to cargo-ament-build
```

#### 2.7: Documentation (Week 4-5)
- [ ] User guide (installation, usage)
- [ ] API documentation (rustdoc)
- [ ] Troubleshooting guide
- [ ] Migration guide (from ros2_rust **and cargo-ament-build**)
- [ ] Video tutorial (optional)

### Phase 2 Deliverables

- [x] Services & actions support
- [x] Smart caching system
- [x] Enhanced CLI
- [x] Performance optimizations
- [x] **Ament installation (cargo-ament-build functionality absorbed)**
- [x] Comprehensive documentation

### Phase 2 Success Metrics

- ✅ Supports all ROS message types (msg, srv, action)
- ✅ Cache hit rate > 95% in typical workflow
- ✅ Cold build < 30s
- ✅ Hot build < 2s
- ✅ **Ament installation produces identical output to cargo-ament-build**
- ✅ **`cargo ros2 ament-build` works as drop-in replacement**
- ✅ Zero configuration for users

---

## Phase 3: colcon Integration & Production

**Duration**: 4-5 weeks
**Goal**: Production-ready, integrated with ROS ecosystem

### Milestones

#### 3.1: colcon-ros-cargo Modification & Fork (Week 1-2)

**Background**: cargo-ros2 now provides complete ament installation (Phase 2.6). We need to modify colcon-ros-cargo to call `cargo ros2` instead of `cargo ament-build`. See `docs/UNIFIED_ARCHITECTURE.md` for details.

**Tasks**:
- [ ] Fork colcon-ros-cargo (in `tmp/colcon-ros-cargo/`)
  - [ ] Create fork on GitHub: `ros2-rust-community/colcon-ros-cargo`
  - [ ] Document fork rationale
- [ ] Modify `colcon_ros_cargo/task/ament_cargo/build.py`
  - [ ] Change detection check from `cargo ament-build --help` to `cargo ros2 --version`
  - [ ] Update error message to recommend `cargo install cargo-ros2`
  - [ ] Modify `_build_cmd()` to call `cargo ros2 ament-build` instead of `cargo ament-build`
  - [ ] Ensure all args are forwarded correctly
  - [ ] Remove cargo-ament-build dependency from setup.cfg
  - [ ] Add cargo-ros2 to documentation
- [ ] Update documentation
  - [ ] README.md with new installation instructions
  - [ ] Migration guide from original colcon-ros-cargo
  - [ ] Compatibility notes
- [ ] Test with existing colcon projects
  - [ ] Test with simple ROS 2 Rust package
  - [ ] Test with workspace containing multiple packages
  - [ ] Test with message dependencies
  - [ ] Verify ament layout is identical
- [ ] Prepare upstream PR
  - [ ] Document changes thoroughly
  - [ ] Include backward compatibility (detect both tools)
  - [ ] Provide deprecation timeline for cargo-ament-build
  - [ ] Submit PR to original colcon-ros-cargo

**Acceptance Criteria**:
```bash
# Install modified colcon-ros-cargo + cargo-ros2
pip install colcon-ros-cargo-v2  # (our fork initially)
cargo install cargo-ros2

# Standard colcon workflow
colcon build --packages-select my_rust_pkg
# → colcon-ros-cargo detects cargo-ros2
# → Calls: cargo ros2 ament-build --install-base ...
# → Generates bindings
# → Builds package
# → Installs with ament layout
# → Success!

# Verify output
ls install/my_rust_pkg/
# lib/my_rust_pkg/         ✓ binaries
# share/my_rust_pkg/rust/  ✓ source
# share/my_rust_pkg/        ✓ package.xml
# share/ament_index/       ✓ markers
```

#### 3.2: Multi-Distro Support (Week 2)
- [ ] Detect ROS_DISTRO
- [ ] Handle distro-specific differences
  - [ ] API changes (Humble vs Iron vs Jazzy)
  - [ ] Path differences
  - [ ] Version mapping
- [ ] Validation for distro conflicts
- [ ] Tests on all supported distros

**Supported Distros** (initial):
- [x] Humble (24.04 LTS)
- [x] Iron (EOL 2024)
- [x] Jazzy (latest)

#### 3.3: IDE Integration (Week 2-3)
- [ ] Generate rust-project.json
  - [ ] Include binding crates
  - [ ] Correct dependency graph
- [ ] Test with rust-analyzer
- [ ] Test with VS Code
- [ ] Test with IntelliJ IDEA

**Acceptance Criteria**:
- ✅ Autocomplete works for ROS types
- ✅ Go-to-definition jumps to generated code
- ✅ No false errors in IDE

#### 3.4: Robustness & Edge Cases (Week 3)
- [ ] Handle missing ROS installation
- [ ] Handle missing packages
- [ ] Handle conflicting distros
- [ ] Handle stale patches
- [ ] Handle circular dependencies
- [ ] Comprehensive error tests

#### 3.5: Security Audit (Week 3-4)
- [ ] Path injection prevention
- [ ] Code injection prevention
- [ ] Validate all user input
- [ ] Security review by external auditor
- [ ] Document security guarantees

#### 3.6: Release Preparation (Week 4-5)
- [ ] Semantic versioning (0.1.0)
- [ ] Publish to crates.io
- [ ] Create GitHub releases
- [ ] Announce on ROS Discourse
- [ ] Submit to Awesome Rust
- [ ] Create landing page (optional)

### Phase 3 Deliverables

- [x] colcon-ros-cargo plugin
- [x] Multi-distro support
- [x] IDE integration
- [x] Comprehensive tests
- [x] Security audit
- [x] Public release (0.1.0)

### Phase 3 Success Metrics

- ✅ Passes all existing ros2_rust tests
- ✅ Works with colcon out-of-the-box
- ✅ Supports Humble, Iron, Jazzy
- ✅ No known security vulnerabilities
- ✅ Community adoption begins

---

## Phase 4: Advanced Features (Future)

**Duration**: TBD (post-1.0)
**Goal**: Advanced features for power users

### Potential Features

#### 4.1: Custom Message Formats
- [ ] Plugin system for message formats
- [ ] Protobuf support
- [ ] FlatBuffers support
- [ ] Custom IDL parsers

#### 4.2: Cross-Compilation
- [ ] ARM target support
- [ ] RISC-V target support
- [ ] Bundled typesupport libs
- [ ] Target-specific ament_index

#### 4.3: Hot Reload
- [ ] Watch mode (cargo ros2 watch)
- [ ] File system watcher
- [ ] Incremental regeneration
- [ ] Auto cargo check

#### 4.4: GUI Tools
- [ ] Visual dependency browser
- [ ] Cache explorer
- [ ] Code diff viewer
- [ ] Performance profiler

#### 4.5: Advanced Optimization
- [ ] Link-time optimization (LTO)
- [ ] Profile-guided optimization (PGO)
- [ ] Monomorphization hints
- [ ] Binary size reduction

---

## Dependencies & Prerequisites

### Development Dependencies

```toml
[dependencies]
# Core
clap = "4.0"           # CLI parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
sha2 = "0.10"          # Checksums

# Code generation
quote = "1.0"          # Rust code generation
proc-macro2 = "1.0"
syn = "2.0"            # Rust parsing (for validation)

# Parsing
pest = "2.0"           # PEG parser generator
pest_derive = "2.0"

# Performance
rayon = "1.0"          # Parallel processing

# Testing
tempfile = "3.0"       # Temp directories for tests
```

### System Dependencies

```bash
# ROS 2 installation (any distro)
sudo apt install ros-humble-desktop

# Rust toolchain
rustup default stable

# Optional: colcon
sudo apt install python3-colcon-common-extensions
```

---

## Testing Strategy

### Unit Tests

- **Discovery**: Mock filesystem, test package finding
- **Parser**: Parse all ROS message syntax variants
- **Generator**: Validate generated code structure
- **Cache**: Test invalidation logic

**Target**: >90% code coverage

### Integration Tests

- **End-to-end**: Generate + compile real ROS packages
- **colcon**: Test within colcon workspace
- **Multi-distro**: Test on Humble, Iron, Jazzy

**Target**: All critical paths covered

### Benchmarks

- **Generation speed**: Messages per second
- **Cache hit rate**: Typical workflow simulation
- **Build time**: Cold vs hot builds

**Target**: Consistently meet performance metrics

---

## Release Schedule

### Alpha Releases (Phase 1)

- `0.1.0-alpha.1`: Basic message support
- `0.1.0-alpha.2`: With caching
- `0.1.0-alpha.3`: Services & actions

**Timeline**: Week 4-8

### Beta Releases (Phase 2)

- `0.1.0-beta.1`: Feature-complete
- `0.1.0-beta.2`: colcon integration
- `0.1.0-beta.3`: Multi-distro

**Timeline**: Week 9-12

### Stable Release (Phase 3)

- `0.1.0`: Production-ready
- Announcement on ROS Discourse
- Publish to crates.io

**Timeline**: Week 13-16

### Maintenance Releases

- `0.1.x`: Bug fixes
- `0.2.0`: New features (Phase 4)
- `1.0.0`: Stable API guarantee

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Cargo compatibility issues | Medium | High | Early testing, upstream engagement |
| ROS API changes | Low | Medium | Multi-distro testing, version detection |
| Performance not meeting targets | Medium | Medium | Early benchmarking, optimization phase |
| colcon integration complexity | High | High | Study existing plugins, incremental approach |

### Resource Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Developer availability | Medium | High | Clear milestones, modular design |
| Community adoption slow | Medium | Medium | Marketing, documentation, examples |
| Maintenance burden | Low | Medium | Clean architecture, good tests |

---

## Success Criteria (Overall)

### Technical Success

- ✅ Solves circular dependency problem
- ✅ Works with system-installed ROS packages
- ✅ Faster than ros2_rust (with cache)
- ✅ Standard Cargo experience
- ✅ Passes all tests

### User Success

- ✅ Positive feedback from early adopters
- ✅ Adoption by ≥3 significant ROS projects
- ✅ 100+ GitHub stars
- ✅ Featured on Awesome Rust

### Ecosystem Success

- ✅ Accepted by ros2_rust maintainers (or)
- ✅ Becomes de facto standard for ROS Rust
- ✅ Integrated into official ROS documentation
- ✅ Used in ROS tutorials

---

## Current Status

**Phase**: 0 (Design) ✅
**Next**: Phase 1.1 (Project Setup)
**Date**: 2025-01-29

---

## Appendix: Task Checklist

### Phase 1 Tasks (Detailed)

```
[ ] Week 1
  [ ] Day 1-2: Repository setup, CI/CD
  [ ] Day 3-5: Discovery module implementation
  [ ] Day 6-7: Discovery tests & documentation

[ ] Week 2
  [ ] Day 1-3: Message parser (lexer + parser)
  [ ] Day 4-5: Parser tests
  [ ] Day 6-7: Code generator foundation

[ ] Week 3
  [ ] Day 1-3: Code generator (struct + FFI)
  [ ] Day 4-5: Binding generator orchestration
  [ ] Day 6-7: Integration tests

[ ] Week 4
  [ ] Day 1-3: CLI implementation
  [ ] Day 4-5: End-to-end testing
  [ ] Day 6-7: Documentation & examples
```

---

**End of Roadmap**

**Last Updated**: 2025-01-29
**Next Review**: After Phase 1 completion
