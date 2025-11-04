// Simple ROS 2 Publisher Example
//
// This example demonstrates:
// - Using cargo-ros2 for automatic binding generation
// - Working with ROS message types
// - Standard Cargo project structure

use std_msgs::msg::String as RosString;

fn main() {
    println!("Simple Publisher Example");
    println!("========================\n");

    // Create a ROS String message
    // cargo-ros2 automatically generates these types from .msg files
    let message = RosString {
        data: "Hello from cargo-ros2!".to_string(),
    };

    println!("Created message:");
    println!("  Type: std_msgs::msg::String");
    println!("  Data: {}", message.data);

    println!("\n✓ Message created successfully!");
    println!("\nNote: This is a minimal example showing binding generation.");
    println!("For full ROS 2 publishing, use the rclrs crate.");
}
