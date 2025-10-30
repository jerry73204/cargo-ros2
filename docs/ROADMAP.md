# Roadmap

## Progress Summary

**Overall Progress**: 4 of 18 subphases complete (22%)

| Phase                                 | Status         | Progress      |
|---------------------------------------|----------------|---------------|
| Phase 0: Project Preparation          | ✅ Complete    | 3/3 subphases |
| Phase 1: Native Rust IDL Generator    | 🔄 In Progress | 1/4 subphases |
| Phase 2: cargo-ros2 Tools             | ⏳ Not Started | 0/2 subphases |
| Phase 3: Production Features          | ⏳ Not Started | 0/4 subphases |
| Phase 4: colcon Integration & Release | ⏳ Not Started | 0/3 subphases |

**Latest Achievement**: IDL Parser complete with 22 passing tests, full support for messages, services, and actions! 🎉

---

## Phase 0: Project Preparation

**Goal**: Set up project structure, tooling, and development infrastructure.

**Duration**: 1 week

### Subphase 0.1: Workspace Setup (3 days) ✅

- [x] Cargo workspace
  - [x] Create workspace root with Cargo.toml
  - [x] Set up crate structure:
    - [x] `cargo-ros2/` - Main CLI tool
    - [x] `cargo-ros2-bindgen/` - Binding generator CLI
    - [x] `rosidl-parser/` - IDL parser library
    - [x] `rosidl-codegen/` - Code generator library
    - [x] `rosidl-runtime-rs/` - Runtime support library
  - [x] Configure workspace Cargo.toml with members
  - [x] Set up .gitignore (target/, .ros2_bindgen_cache, etc.)

- [x] Cargo profiles
  - [x] Add `dev-release` profile in workspace Cargo.toml
  - [x] Inherit from release profile
  - [x] Enable debug assertions (`debug-assertions = true`)
  - [x] Enable debug info (`debug = true`)
  - [x] Profile used for testing and linting

- [x] Makefile (minimalist style)
  - [x] Each non-file target preceded by `.PHONY`
  - [x] `make build` - Build with dev-release profile
  - [x] `make test` - Run `cargo nextest run --no-fail-fast --cargo-profile dev-release`
  - [x] `make clean` - Clean all artifacts
  - [x] `make format` - Run `cargo +nightly fmt`
  - [x] `make lint` - Run `cargo clippy --profile dev-release -- -D warnings`
  - [x] `make check` - Run `cargo check --profile dev-release`
  - [x] `make doc` - Generate documentation
  - [x] `make install` - Install binaries

- [x] Testing
  - [x] Run `make format && make lint` to verify code quality
  - [x] Verify all Makefile targets work
  - [x] Test workspace builds with dev-release profile

**Acceptance**:
```bash
make build              # All crates compile with dev-release
make format && make lint  # Code is formatted and passes clippy
make test               # All tests pass with nextest
```

### Subphase 0.2: Documentation Setup (2 days) ✅

- [x] Documentation structure
  - [x] Update README.md
  - [x] Create CONTRIBUTING.md
  - [x] Add Code of Conduct
  - [x] Create issue templates

- [x] Testing
  - [x] Verify documentation renders correctly
  - [x] Check all links work

**Acceptance**:
- Documentation is complete and renders correctly ✅

### Subphase 0.3: Dependencies & Tooling (2 days) ✅

- [x] Add core dependencies
  - [x] clap (CLI parsing)
  - [x] eyre (error handling)
  - [x] serde, serde_json (serialization)
  - [x] cargo-manifest (Cargo.toml parsing)
  - [x] toml (config file handling)
  - [x] sha2 (caching checksums)

- [x] Development tools
  - [x] Install cargo-nextest (required for testing)
  - [x] Install Rust nightly (required for formatting)
  - [x] cargo-watch (optional, for development)
  - [x] cargo-deny (dependency checks)

- [x] Testing
  - [x] Verify all dependencies compile
  - [x] Run `make test` with cargo-nextest
  - [x] Run `make format && make lint` successfully

---

## Phase 1: Native Rust IDL Generator

**Goal**: Implement pure Rust parser and code generator for ROS IDL files (.msg, .srv, .action).

**Duration**: 4 weeks

### Subphase 1.1: IDL Parser - Messages (2 weeks) ✅

- [x] Lexer
  - [x] Tokenize .msg files
  - [x] Handle comments
  - [x] Parse field types (primitives, arrays, bounded strings)
  - [x] Parse constants

- [x] Parser
  - [x] Build AST for .msg files
  - [x] Type resolution (built-in types)
  - [x] Dependency resolution (other message types)
  - [x] Validation (field names, types)

- [x] Unit tests
  - [x] Test lexer with various .msg formats (10 tests)
  - [x] Test parser with std_msgs examples (14 tests)
  - [x] Test error handling (invalid syntax)
  - [x] Test edge cases (empty messages, comments)

- [x] Integration tests
  - [x] Parse all message types (primitives, arrays, sequences, strings)
  - [x] Verify AST correctness
  - [x] Service and action parsing implemented

**Acceptance**:
```bash
cargo test --package rosidl-parser
# → Summary [0.011s] 22 tests run: 22 passed, 0 skipped ✅
```

### Subphase 1.2: Code Generator - Messages (2 weeks)

- [ ] Template system
  - [ ] Design template format (or use tera/askama)
  - [ ] Port msg.rs.em logic to Rust templates
  - [ ] Generate struct definitions
  - [ ] Generate serialization/deserialization code
  - [ ] Generate trait implementations (Default, Debug, Clone)

- [ ] Cargo.toml generation
  - [ ] Generate package manifest
  - [ ] Add dependencies (rosidl_runtime_rs)
  - [ ] Handle transitive deps

- [ ] build.rs generation
  - [ ] Link ROS typesupport libraries
  - [ ] Handle pkg-config integration

- [ ] Unit tests
  - [ ] Test code generation for simple messages
  - [ ] Test generated code compiles
  - [ ] Test generated code has correct API
  - [ ] Test serialization round-trips

- [ ] Integration tests
  - [ ] Generate bindings for std_msgs
  - [ ] Compile generated code
  - [ ] Compare output with rosidl_generator_rs
  - [ ] Test FFI compatibility

**Acceptance**:
```bash
cargo test --package rosidl-codegen
# → Generates std_msgs bindings
# → Generated code compiles
# → API matches rosidl_generator_rs output
```

### Subphase 1.3: Services & Actions Support (1 week)

- [ ] Service parser
  - [ ] Parse .srv files (request/response)
  - [ ] Handle embedded message definitions
  - [ ] Validate service structure

- [ ] Action parser
  - [ ] Parse .action files (goal/result/feedback)
  - [ ] Handle three-part structure

- [ ] Code generation
  - [ ] Generate service types
  - [ ] Generate action types
  - [ ] Generate client/server stubs

- [ ] Unit tests
  - [ ] Test .srv parsing
  - [ ] Test .action parsing
  - [ ] Test generated service code
  - [ ] Test generated action code

- [ ] Integration tests
  - [ ] Generate example_interfaces services
  - [ ] Generate action_msgs actions
  - [ ] Verify generated code compiles

**Acceptance**:
```bash
cargo test --package rosidl-codegen -- services
cargo test --package rosidl-codegen -- actions
# → All service/action types generate correctly
```

### Subphase 1.4: Parity Testing (1 week)

- [ ] Comprehensive parity tests
  - [ ] Compare output with rosidl_generator_rs for all ROS packages
  - [ ] Test with common_interfaces (std_msgs, sensor_msgs, geometry_msgs, etc.)
  - [ ] Test with action_msgs, example_interfaces
  - [ ] Document any intentional differences

- [ ] Performance testing
  - [ ] Benchmark generation speed vs Python generator
  - [ ] Profile and optimize hot paths
  - [ ] Target: ≥2x faster than Python

- [ ] Edge case testing
  - [ ] Test with complex nested messages
  - [ ] Test with large arrays
  - [ ] Test with unusual naming
  - [ ] Test with Unicode in comments

**Acceptance**:
```bash
cargo test --package rosidl-codegen -- parity
# → All common_interfaces packages generate identical API
# → Generation is ≥2x faster than rosidl_generator_rs
```

---

## Phase 2: cargo-ros2 Tools

**Goal**: Build cargo-ros2-bindgen and cargo-ros2 tools using native generator.

**Duration**: 4 weeks

### Subphase 2.1: cargo-ros2-bindgen (2 weeks)

- [ ] Ament index integration
  - [ ] Parse AMENT_PREFIX_PATH
  - [ ] Implement package discovery
  - [ ] Find package share directories
  - [ ] Locate .msg/.srv/.action files

- [ ] Generator integration
  - [ ] Invoke native Rust generator
  - [ ] Handle package dependencies
  - [ ] Generate output to specified directory
  - [ ] Post-process generated files

- [ ] CLI
  - [ ] `--package <name>` flag
  - [ ] `--output <path>` flag
  - [ ] `--package-path <path>` flag (for local packages)
  - [ ] Error handling and reporting

- [ ] Unit tests
  - [ ] Test ament_index parsing
  - [ ] Test package discovery
  - [ ] Test path handling
  - [ ] Test error cases (missing packages)

- [ ] Integration tests
  - [ ] Generate std_msgs from system installation
  - [ ] Generate sensor_msgs with dependencies
  - [ ] Verify output structure
  - [ ] Verify generated code compiles

**Acceptance**:
```bash
cargo-ros2-bindgen --package std_msgs --output target/test/std_msgs
cargo build --manifest-path target/test/std_msgs/Cargo.toml
# → Bindings generate and compile successfully
```

### Subphase 2.2: cargo-ros2 Core (2 weeks)

- [ ] ROS dependency discovery
  - [ ] Parse Cargo.toml dependencies
  - [ ] Check against ament_index
  - [ ] Parse package.xml for transitive deps
  - [ ] Recursively discover dep tree
  - [ ] Filter for interface packages only
  - [ ] Detect cycles in dependency graph

- [ ] Cache system
  - [ ] SHA256 checksum calculation for .msg/.srv/.action files
  - [ ] .ros2_bindgen_cache file format (JSON)
  - [ ] Cache hit/miss logic
  - [ ] Cache invalidation (package version, ROS_DISTRO)
  - [ ] Update cache on generation

- [ ] Cargo config patcher
  - [ ] Read/write .cargo/config.toml
  - [ ] Add [patch.crates-io] entries
  - [ ] Preserve existing user config
  - [ ] Handle merge conflicts

- [ ] Main workflow
  - [ ] Discover ROS deps
  - [ ] Check cache for each package
  - [ ] Generate missing/stale bindings
  - [ ] Patch .cargo/config.toml
  - [ ] Invoke cargo build

- [ ] CLI
  - [ ] `cargo ros2 build` command
  - [ ] `cargo ros2 check` command
  - [ ] `--bindings-only` flag
  - [ ] Forward args to cargo (after `--`)

- [ ] Unit tests
  - [ ] Test dependency discovery
  - [ ] Test transitive dep resolution
  - [ ] Test cache hit/miss logic
  - [ ] Test checksum calculation
  - [ ] Test config patching
  - [ ] Test workspace detection

- [ ] Integration tests
  - [ ] End-to-end test with mock project
  - [ ] Test cache behavior (warm/cold)
  - [ ] Test with multiple packages
  - [ ] Test workspace scenarios
  - [ ] Test error recovery

**Acceptance**:
```bash
# Create test project
cargo new test-project
cd test-project
echo 'std_msgs = "*"' >> Cargo.toml

cargo ros2 build
# → Discovers std_msgs
# → Generates bindings
# → Caches results
# → Builds successfully

cargo ros2 build
# → Cache hit, no regeneration
# → Builds in <5s
```

---

## Phase 3: Production Features

**Goal**: Add services, actions, ament installation, performance optimizations.

**Duration**: 5 weeks

### Subphase 3.1: Services & Actions Integration (1 week)

- [ ] Full service support in cargo-ros2-bindgen
  - [ ] Detect .srv files in packages
  - [ ] Generate service bindings
  - [ ] Handle service dependencies

- [ ] Full action support in cargo-ros2-bindgen
  - [ ] Detect .action files in packages
  - [ ] Generate action bindings
  - [ ] Handle action dependencies

- [ ] Unit tests
  - [ ] Test service discovery
  - [ ] Test action discovery
  - [ ] Test dependency resolution

- [ ] Integration tests
  - [ ] Generate example_interfaces (services)
  - [ ] Generate action_tutorials_interfaces
  - [ ] Compile and test generated code

**Acceptance**:
```rust
// In user project
use example_interfaces::srv::AddTwoInts;
use action_tutorials_interfaces::action::Fibonacci;

let req = AddTwoInts::Request { a: 1, b: 2 };
let goal = Fibonacci::Goal { order: 5 };
// Compiles successfully
```

### Subphase 3.2: Ament Installation (2 weeks)

- [ ] Extract from cargo-ament-build
  - [ ] Study marker creation logic
  - [ ] Study source installation logic
  - [ ] Study binary installation logic
  - [ ] Study metadata installation logic

- [ ] Implement AmentInstaller module
  - [ ] `create_markers()` - Create ament_index markers
  - [ ] `install_source()` - Install source files
  - [ ] `install_binaries()` - Install compiled binaries
  - [ ] `install_metadata()` - Install package.xml, etc.
  - [ ] Handle library vs binary packages

- [ ] Implement `cargo ros2 ament-build` command
  - [ ] Parse `--install-base <path>` arg
  - [ ] Run full workflow (generate → build → install)
  - [ ] Forward cargo args after `--`
  - [ ] Detect pure library packages (use `cargo check`)

- [ ] Unit tests
  - [ ] Test marker generation
  - [ ] Test path construction
  - [ ] Test file copying logic
  - [ ] Test pure library detection

- [ ] Integration tests
  - [ ] Compare output with cargo-ament-build
  - [ ] Test with binary package
  - [ ] Test with library package
  - [ ] Test metadata installation
  - [ ] Verify ament_index correctness

**Acceptance**:
```bash
cargo ros2 ament-build --install-base install/my_pkg -- --release
ls install/my_pkg/lib/my_pkg/       # binaries (if any)
ls install/my_pkg/share/my_pkg/rust/  # source files
ls install/my_pkg/share/ament_index/  # markers
# → Directory structure matches cargo-ament-build exactly
```

### Subphase 3.3: Performance & CLI Polish (1 week)

- [ ] Parallel generation
  - [ ] Use rayon for parallel package generation
  - [ ] Parallelize checksum calculation
  - [ ] Optimize cache lookups

- [ ] Better error messages
  - [ ] Detailed error context
  - [ ] Suggestions for common issues
  - [ ] Pretty error formatting (miette?)

- [ ] Progress indicators
  - [ ] Show generation progress
  - [ ] Show build progress
  - [ ] Estimated time remaining

- [ ] CLI improvements
  - [ ] `cargo ros2 cache list` - Show cached bindings
  - [ ] `cargo ros2 cache rebuild <pkg>` - Force regeneration
  - [ ] `cargo ros2 cache clean` - Clear cache
  - [ ] `cargo ros2 info <package>` - Show package info
  - [ ] `--verbose` flag for debugging

- [ ] Performance tests
  - [ ] Benchmark cold build time
  - [ ] Benchmark hot build time
  - [ ] Benchmark cache operations
  - [ ] Profile and optimize bottlenecks

**Target**: Cold build <60s, Hot build <5s (for typical projects)

### Subphase 3.4: Testing & Documentation (2 weeks)

- [ ] Comprehensive unit tests
  - [ ] Achieve >80% code coverage
  - [ ] Test all error paths
  - [ ] Test edge cases
  - [ ] Add property-based tests (proptest)

- [ ] Integration tests
  - [ ] Real-world project tests
  - [ ] Test with multiple ROS distros (Humble, Iron, Jazzy)
  - [ ] Test workspace scenarios
  - [ ] Test cross-crate dependencies
  - [ ] Test with large dependency trees

- [ ] Documentation
  - [ ] User guide (getting started, installation)
  - [ ] CLI reference (all commands and flags)
  - [ ] Architecture documentation (how it works)
  - [ ] API docs (rustdoc for all public APIs)
  - [ ] Examples (simple subscriber, publisher, service)
  - [ ] Troubleshooting guide (common errors)
  - [ ] Migration guide (from ros2_rust)

- [ ] End-to-end tests
  - [ ] Test complete workflow from empty project to running binary
  - [ ] Test colcon-like workflows
  - [ ] Test failure recovery

**Acceptance**:
```bash
cargo test --all
# → All tests pass
# → Coverage >80%

cargo doc --all --no-deps
# → Documentation builds without warnings

make lint
# → No clippy warnings
```

---

## Phase 4: colcon Integration & Release

**Goal**: Seamless colcon integration and public release.

**Duration**: 4 weeks

### Subphase 4.1: colcon-ros-cargo Fork (2 weeks)

- [ ] Fork colcon-ros-cargo
  - [ ] Create fork repository
  - [ ] Document all changes

- [ ] Modify build.py
  - [ ] Detect cargo-ros2: `cargo ros2 --version`
  - [ ] Change command: `cargo ros2 ament-build --install-base ...`
  - [ ] Remove cargo-ament-build dependency
  - [ ] Update error messages
  - [ ] Handle edge cases (missing cargo-ros2)

- [ ] Compatibility layer
  - [ ] Support same arguments as cargo-ament-build
  - [ ] Maintain same output format
  - [ ] Preserve colcon integration points

- [ ] Unit tests
  - [ ] Test detection logic
  - [ ] Test command construction
  - [ ] Test argument forwarding

- [ ] Integration tests
  - [ ] Test with simple Rust package
  - [ ] Test with workspace (multiple packages)
  - [ ] Test with message dependencies
  - [ ] Compare output with original colcon-ros-cargo
  - [ ] Test build order in workspace

**Acceptance**:
```bash
# Install fork
pip install git+https://github.com/user/colcon-ros-cargo.git

# Build with colcon
colcon build --packages-select my_rust_pkg
# → Detects cargo-ros2
# → Builds successfully
# → Output identical to cargo-ament-build workflow
```

### Subphase 4.2: Multi-Distro Support (1 week)

- [ ] ROS distro detection
  - [ ] Read ROS_DISTRO environment variable
  - [ ] Validate against supported distros
  - [ ] Warn on unknown distros

- [ ] Handle distro differences
  - [ ] Test on Humble (Ubuntu 22.04)
  - [ ] Test on Iron (Ubuntu 22.04)
  - [ ] Test on Jazzy (Ubuntu 24.04)
  - [ ] Document distro-specific issues

- [ ] Integration tests
  - [ ] Test common_interfaces on all distros
  - [ ] Test version compatibility
  - [ ] Test package discovery across distros

**Acceptance**:
```bash
# On Humble
ROS_DISTRO=humble cargo ros2 build
# → Works correctly

# On Jazzy
ROS_DISTRO=jazzy cargo ros2 build
# → Works correctly
```

### Subphase 4.3: Release Preparation (1 week)

- [ ] Final testing
  - [ ] Full test suite on all distros
  - [ ] Real-world project testing
  - [ ] Performance benchmarks
  - [ ] Memory profiling

- [ ] Security audit
  - [ ] Run cargo-deny
  - [ ] Check dependencies for vulnerabilities
  - [ ] Review unsafe code (if any)
  - [ ] Add security policy

- [ ] Documentation review
  - [ ] Proofread all docs
  - [ ] Verify examples work
  - [ ] Create changelog

- [ ] Release process
  - [ ] Publish rosidl-runtime-rs to crates.io
  - [ ] Publish cargo-ros2-bindgen to crates.io
  - [ ] Publish cargo-ros2 to crates.io
  - [ ] Create GitHub release v0.1.0
  - [ ] Tag release commit
  - [ ] Generate release notes

- [ ] Community announcement
  - [ ] Post on ROS Discourse
  - [ ] Share on ros2_rust GitHub discussions
  - [ ] Update ros2_rust documentation (if accepted)
  - [ ] Create example repositories

**Acceptance**:
```bash
# Users can install from crates.io
cargo install cargo-ros2

# Users can build ROS 2 Rust projects
cargo ros2 build
# → Works out of the box
```

---

## Milestones

### M0: Project Ready (End of Phase 0)
- Workspace structure in place (5 crates)
- dev-release profile configured
- Makefile with all targets working
- cargo-nextest and nightly Rust installed
- `make format && make lint` passes
- Development environment ready

### M1: Native Generator Complete (End of Phase 1)
- Pure Rust IDL parser working (Subphase 1.1)
- Code generation for messages (Subphase 1.2)
- Services & actions support (Subphase 1.3)
- Parity with rosidl_generator_rs (Subphase 1.4)
- No Python dependency

### M2: Tools Complete (End of Phase 2)
- cargo-ros2-bindgen functional (Subphase 2.1)
- cargo-ros2 build workflow working (Subphase 2.2)
- Caching system operational
- Core functionality proven

### M3: Feature Complete (End of Phase 3)
- Full service/action support (Subphase 3.1)
- Ament installation integrated (Subphase 3.2)
- Performance optimized (Subphase 3.3)
- Comprehensive testing & docs (Subphase 3.4)

### M4: Production Ready (End of Phase 4)
- colcon integration working (Subphase 4.1)
- Multi-distro support verified (Subphase 4.2)
- Public release 0.1.0 (Subphase 4.3)
- Community adoption begins

---

## Success Criteria

### Technical
- [ ] Generates bindings for all ROS interface packages
- [ ] Pure Rust implementation (no Python dependency)
- [ ] Passes all tests (unit, integration, end-to-end)
- [ ] Test coverage >80%
- [ ] No performance regression vs cargo-ament-build
- [ ] Cold build <60s, hot build <5s
- [ ] Works with Humble, Iron, Jazzy
- [ ] Compatible with existing ros2_rust ecosystem

### Quality
- [ ] Zero clippy warnings
- [ ] All public APIs documented
- [ ] Comprehensive user guide
- [ ] Example projects available
- [ ] Security audit passed

### Community
- [ ] Positive feedback from ros2-rust maintainers
- [ ] Adoption by ≥3 real-world projects
- [ ] colcon-ros-cargo PR accepted or fork widely used
- [ ] Active issue resolution
- [ ] Clear contribution guidelines

---

## Testing Strategy

### Unit Tests (Per Component)
- Test individual functions/modules in isolation
- Mock external dependencies (filesystem, ament_index)
- Test error paths and edge cases
- Use property-based testing where applicable
- Target: >80% coverage per crate

### Integration Tests (Per Phase)
- Test component interactions
- Use real ROS packages (std_msgs, sensor_msgs, etc.)
- Test against actual ament installations
- Verify generated code compiles and links
- Test caching behavior
- Test workspace scenarios

### End-to-End Tests (Per Milestone)
- Complete workflow from empty project to running binary
- Test with real ROS distros (Docker-based)
- Performance benchmarks
- Comparison with existing tools (cargo-ament-build, rosidl_generator_rs)
- User scenario testing

### Regression Tests (Continuous)
- Test against known issues from ros2_rust
- Test with complex dependency graphs
- Test with large codebases

---

## Current Status

**Phase**: Phase 1, Subphase 1.1 Complete ✅
**Completed**:
- ✅ Phase 0 Complete (all subphases)
- ✅ Subphase 1.1: IDL Parser - Messages

**Next**: Phase 1, Subphase 1.2 (Code Generator - Messages)
**Date**: 2025-10-31

---

## Timeline Summary

| Phase                                 | Duration | Cumulative | Milestone              |
|---------------------------------------|----------|------------|------------------------|
| Phase 0: Project Preparation          | 1 week   | 1 week     | M0: Project Ready      |
| Phase 1: Native Rust IDL Generator    | 4 weeks  | 5 weeks    | M1: Generator Complete |
| Phase 2: cargo-ros2 Tools             | 4 weeks  | 9 weeks    | M2: Tools Complete     |
| Phase 3: Production Features          | 5 weeks  | 14 weeks   | M3: Feature Complete   |
| Phase 4: colcon Integration & Release | 4 weeks  | 18 weeks   | M4: Production Ready   |

**Total Duration**: 18 weeks (~4.5 months)

**Note**: This is more ambitious than the original 12-16 week timeline, but includes:
- Complete native Rust implementation (no Python dependency)
- Comprehensive testing at every phase
- Better tooling (Makefile, enhanced CLI)
- More thorough documentation
