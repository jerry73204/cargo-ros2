// Simple ROS 2 Subscriber Example
//
// This example demonstrates:
// - Using cargo-ros2 with multiple ROS packages
// - Working with different message types
// - Default trait implementation

use geometry_msgs::msg::{Point, Pose};
use std_msgs::msg::String as RosString;

fn main() {
    println!("Simple Subscriber Example");
    println!("=========================\n");

    // Example 1: String message
    let string_msg = RosString {
        data: "Received message".to_string(),
    };

    println!("String Message:");
    println!("  Type: std_msgs::msg::String");
    println!("  Data: {}\n", string_msg.data);

    // Example 2: Point message with default values
    let point = Point::default();

    println!("Point Message (default):");
    println!("  Type: geometry_msgs::msg::Point");
    println!("  x: {}, y: {}, z: {}\n", point.x, point.y, point.z);

    // Example 3: Point with custom values
    let custom_point = Point {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };

    println!("Point Message (custom):");
    println!("  Type: geometry_msgs::msg::Point");
    println!("  x: {}, y: {}, z: {}\n", custom_point.x, custom_point.y, custom_point.z);

    // Example 4: Nested message (Pose contains Point and Quaternion)
    let pose = Pose {
        position: custom_point,
        orientation: geometry_msgs::msg::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
    };

    println!("Pose Message:");
    println!("  Type: geometry_msgs::msg::Pose");
    println!("  Position: ({}, {}, {})", pose.position.x, pose.position.y, pose.position.z);
    println!("  Orientation: ({}, {}, {}, {})",
        pose.orientation.x, pose.orientation.y, pose.orientation.z, pose.orientation.w);

    println!("\n✓ All messages created successfully!");
    println!("\nNote: This example shows binding generation for multiple packages.");
    println!("For full ROS 2 subscription, use the rclrs crate.");
}
