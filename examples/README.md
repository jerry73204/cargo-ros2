# cargo-ros2 Examples

This directory contains example projects demonstrating cargo-ros2 usage.

## Prerequisites

- ROS 2 (Humble, Iron, or Jazzy) installed and sourced
- Rust 1.70+ installed
- cargo-ros2 built and available in PATH

```bash
# Source ROS
source /opt/ros/humble/setup.bash

# Verify cargo-ros2 is available
cargo ros2 --version
```

## Examples

### 1. simple_publisher

Basic ROS 2 publisher that sends string messages.

**Features**:
- Uses `std_msgs` package
- Demonstrates automatic binding generation
- Shows project setup with cargo-ros2

**Build and run**:
```bash
cd simple_publisher
cargo ros2 build
cargo run
```

### 2. simple_subscriber

Basic ROS 2 subscriber that receives string messages.

**Features**:
- Uses `std_msgs` package
- Complements simple_publisher
- Demonstrates message reception

**Build and run**:
```bash
cd simple_subscriber
cargo ros2 build
cargo run
```

### Running Together

In separate terminals:

```bash
# Terminal 1: Publisher
cd simple_publisher
cargo ros2 build
cargo run

# Terminal 2: Subscriber
cd simple_subscriber
cargo ros2 build
cargo run
```

## Building Examples

Each example is a standalone Cargo project:

```bash
# Navigate to example
cd simple_publisher

# Build with cargo-ros2 (generates bindings automatically)
cargo ros2 build

# Or just check without building
cargo ros2 check

# Run the example
cargo run

# Install to ament layout
cargo ros2 ament-build --install-base install/simple_publisher --release
```

## Example Structure

All examples follow this structure:

```
example_name/
├── Cargo.toml       # Standard Cargo manifest with ROS dependencies
├── src/
│   └── main.rs      # Example code
├── .gitignore       # Ignores target/, .ros2_bindgen_cache
└── README.md        # Example-specific documentation
```

## Common Patterns

### Adding ROS Dependencies

```toml
[dependencies]
std_msgs = "*"
sensor_msgs = "*"
geometry_msgs = "*"
```

### Using Generated Bindings

```rust
// Import ROS message types (automatic!)
use std_msgs::msg::String as RosString;

fn main() {
    let msg = RosString {
        data: "Hello, ROS 2!".to_string(),
    };

    println!("Message: {}", msg.data);
}
```

## Troubleshooting

### "Failed to load ament index"

Make sure ROS is sourced:
```bash
source /opt/ros/humble/setup.bash
```

### "Package 'foo' not found"

Install the ROS package:
```bash
sudo apt install ros-humble-foo
```

### Stale bindings

Clean and rebuild:
```bash
cargo ros2 clean
cargo ros2 build
```

## Next Steps

- Explore the [CLI Reference](../docs/CLI_REFERENCE.md) for all commands
- Read the [User Guide](../docs/USER_GUIDE.md) for detailed tutorials
- Check the [Design Documentation](../docs/DESIGN.md) for architecture details

---

**Note**: These examples are minimal demonstrations. For production ROS 2 Rust development, see the [rclrs documentation](https://github.com/ros2-rust/rclrs).
