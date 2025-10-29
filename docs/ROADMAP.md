# Roadmap

## Phase 1: MVP (4 weeks)

**Goal**: Generate Rust bindings for ROS messages and build projects.

### Week 1-2: cargo-ros2-bindgen

- [ ] Project setup
  - [ ] Create cargo-ros2-bindgen crate
  - [ ] Add dependencies (clap, anyhow, serde_json)
  - [ ] Set up CI (fmt, clippy, test)

- [ ] Ament index integration
  - [ ] Parse AMENT_PREFIX_PATH
  - [ ] Implement package discovery
  - [ ] Find package share directories

- [ ] Python generator invocation
  - [ ] Shell out to `rosidl_generator_rs`
  - [ ] Pass correct arguments
  - [ ] Handle errors

- [ ] Post-processing
  - [ ] Fix generated Cargo.toml paths
  - [ ] Verify output structure

- [ ] Testing
  - [ ] Unit tests for discovery
  - [ ] Integration test with std_msgs
  - [ ] Verify generated code compiles

**Acceptance**:
```bash
cargo-ros2-bindgen --package std_msgs --output target/ros2_bindings/std_msgs
ls target/ros2_bindings/std_msgs/rust/Cargo.toml  # exists
```

### Week 3-4: cargo-ros2

- [ ] ROS dependency discovery
  - [ ] Parse Cargo.toml dependencies
  - [ ] Check against ament_index
  - [ ] Recursively discover transitive deps from package.xml
  - [ ] Filter for interface packages only

- [ ] Cache system
  - [ ] SHA256 checksum calculation
  - [ ] .ros2_bindgen_cache file format
  - [ ] Cache hit/miss logic
  - [ ] Update cache on generation

- [ ] Cargo config patcher
  - [ ] Read/write .cargo/config.toml
  - [ ] Add [patch.crates-io] entries
  - [ ] Handle existing config

- [ ] Main workflow
  - [ ] Discover ROS deps
  - [ ] Generate bindings (cache-aware)
  - [ ] Patch config
  - [ ] Invoke cargo build

- [ ] CLI
  - [ ] `cargo ros2 build`
  - [ ] `--bindings-only` flag
  - [ ] `--release` flag

- [ ] Testing
  - [ ] End-to-end test with real project
  - [ ] Test cache behavior
  - [ ] Test with multiple packages

**Acceptance**:
```bash
# User project with:
[dependencies]
std_msgs = "*"
sensor_msgs = "*"

cargo ros2 build
# → Generates bindings for std_msgs, sensor_msgs
# → Patches .cargo/config.toml
# → cargo build succeeds
```

---

## Phase 2: Production Features (5 weeks)

**Goal**: Services, actions, caching, ament installation.

### Week 5: Services & Actions

- [ ] Extend cargo-ros2-bindgen
  - [ ] Support .srv files
  - [ ] Support .action files
  - [ ] Test with example_interfaces

**Acceptance**:
```rust
use example_interfaces::srv::AddTwoInts;
let req = AddTwoInts::Request { a: 1, b: 2 };
```

### Week 6-7: Ament Installation

- [ ] Extract from cargo-ament-build
  - [ ] Copy marker creation logic
  - [ ] Copy source installation logic
  - [ ] Copy binary installation logic
  - [ ] Copy metadata installation logic

- [ ] Implement AmentInstaller
  - [ ] create_markers()
  - [ ] install_source()
  - [ ] install_binaries()
  - [ ] install_metadata()

- [ ] Implement `cargo ros2 ament-build`
  - [ ] Parse --install-base arg
  - [ ] Run full workflow (generate → build → install)
  - [ ] Forward cargo args after `--`

- [ ] Testing
  - [ ] Compare output with cargo-ament-build
  - [ ] Test with real ROS package
  - [ ] Test metadata installation

**Acceptance**:
```bash
cargo ros2 ament-build --install-base install/my_pkg -- --release
ls install/my_pkg/lib/my_pkg/       # binaries
ls install/my_pkg/share/my_pkg/rust/  # source
ls install/my_pkg/share/ament_index/  # markers
```

### Week 8: Performance & Polish

- [ ] Parallel generation (rayon)
- [ ] Optimize checksum calculation
- [ ] Better error messages
- [ ] Progress indicators
- [ ] Documentation

- [ ] CLI improvements
  - [ ] `cargo ros2 cache --list`
  - [ ] `cargo ros2 cache --rebuild`
  - [ ] `cargo ros2 info <package>`

**Target**: Cold build < 60s, Hot build < 5s

### Week 9: Testing & Documentation

- [ ] Comprehensive tests
  - [ ] Unit tests (>80% coverage)
  - [ ] Integration tests
  - [ ] Real-world project tests

- [ ] Documentation
  - [ ] User guide
  - [ ] API docs (rustdoc)
  - [ ] Examples
  - [ ] Troubleshooting guide

---

## Phase 3: colcon Integration (4 weeks)

**Goal**: Seamless colcon integration, ready for community use.

### Week 10-11: colcon-ros-cargo Fork

- [ ] Fork colcon-ros-cargo
  - [ ] Create fork repo
  - [ ] Document changes

- [ ] Modify build.py
  - [ ] Change detection: `cargo ros2 --version`
  - [ ] Change command: `cargo ros2 ament-build`
  - [ ] Remove cargo-ament-build dep
  - [ ] Update error messages

- [ ] Testing
  - [ ] Test with simple package
  - [ ] Test with workspace (multiple packages)
  - [ ] Test with message dependencies
  - [ ] Compare output with original

**Acceptance**:
```bash
pip install colcon-ros-cargo-v2
cargo install cargo-ros2

colcon build --packages-select my_rust_pkg
# → Uses cargo-ros2 automatically
# → Output identical to cargo-ament-build
```

### Week 12: Multi-Distro Support

- [ ] ROS_DISTRO detection
- [ ] Handle distro differences
  - [ ] Humble (24.04 LTS)
  - [ ] Iron
  - [ ] Jazzy

- [ ] Testing on all distros

### Week 13: Release Prep

- [ ] Final testing
- [ ] Security audit
- [ ] Performance benchmarks
- [ ] Documentation review

- [ ] Release
  - [ ] Publish to crates.io (cargo-ros2, cargo-ros2-bindgen)
  - [ ] GitHub release (0.1.0)
  - [ ] Announce on ROS Discourse

---

## Phase 4: Native Rust Generator (Future)

**Goal**: Remove Python dependency, full native implementation.

- [ ] IDL parser
  - [ ] Lex .msg/.srv/.action files
  - [ ] Build AST
  - [ ] Type resolution

- [ ] Code generator
  - [ ] Replace EmPy templates with Rust templates
  - [ ] Generate identical output to rosidl_generator_rs
  - [ ] Ensure compatibility

- [ ] Testing
  - [ ] Parity tests with Python generator
  - [ ] All existing tests pass

**Benefit**: Faster generation, no Python dependency

---

## Milestones

### M1: MVP Complete (Week 4)
- cargo-ros2-bindgen works
- cargo-ros2 build works
- Cache system functional

### M2: Feature Complete (Week 9)
- Services & actions
- Ament installation
- Performance optimized

### M3: Production Ready (Week 13)
- colcon integration
- Multi-distro support
- Public release 0.1.0

---

## Success Criteria

### Technical
- [ ] Generates bindings for all ROS interface packages
- [ ] Passes all tests
- [ ] No performance regression vs cargo-ament-build
- [ ] Works with Humble, Iron, Jazzy

### Community
- [ ] Positive feedback from ros2-rust maintainers
- [ ] Adoption by ≥3 projects
- [ ] colcon-ros-cargo PR accepted or fork widely used

---

## Current Status

**Phase**: Design Complete ✅
**Next**: Phase 1, Week 1-2 (cargo-ros2-bindgen)
**Date**: 2025-01-30
