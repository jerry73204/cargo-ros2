// Demonstrates usage of BOTH standard ROS types and custom interface types
fn main() {
    println!("=== Robot Controller Node ===\n");

    // ============================================
    // STANDARD ROS MESSAGE TYPES
    // ============================================
    println!("--- Standard ROS Messages ---");

    // std_msgs
    let string_msg = std_msgs::msg::String::default();
    println!("std_msgs::String: {:?}", string_msg);

    let header = std_msgs::msg::Header::default();
    println!("std_msgs::Header: {:?}", header);

    let bool_msg = std_msgs::msg::Bool::default();
    println!("std_msgs::Bool: {:?}", bool_msg);

    // geometry_msgs
    let point = geometry_msgs::msg::Point::default();
    println!("geometry_msgs::Point: {:?}", point);

    let pose = geometry_msgs::msg::Pose::default();
    println!("geometry_msgs::Pose: {:?}", pose);

    let twist = geometry_msgs::msg::Twist::default();
    println!("geometry_msgs::Twist: {:?}", twist);

    // sensor_msgs
    let imu = sensor_msgs::msg::Imu::default();
    println!("sensor_msgs::Imu: {:?}", imu);

    let laser_scan = sensor_msgs::msg::LaserScan::default();
    println!("sensor_msgs::LaserScan: {:?}", laser_scan);

    // ============================================
    // CUSTOM INTERFACE TYPES
    // ============================================
    println!("\n--- Custom Interface Messages ---");

    // Custom messages
    let status = robot_interfaces::msg::RobotStatus::default();
    println!("robot_interfaces::RobotStatus: {:?}", status);

    let reading = robot_interfaces::msg::SensorReading::default();
    println!("robot_interfaces::SensorReading: {:?}", reading);

    // Custom service types
    println!("\n--- Custom Service Types ---");
    let service_req = robot_interfaces::srv::SetModeRequest::default();
    println!("SetModeRequest: {:?}", service_req);

    let service_resp = robot_interfaces::srv::SetModeResponse::default();
    println!("SetModeResponse: {:?}", service_resp);

    // Custom action types
    println!("\n--- Custom Action Types ---");
    let goal = robot_interfaces::action::NavigateGoal::default();
    println!("NavigateGoal: {:?}", goal);

    let result = robot_interfaces::action::NavigateResult::default();
    println!("NavigateResult: {:?}", result);

    let feedback = robot_interfaces::action::NavigateFeedback::default();
    println!("NavigateFeedback: {:?}", feedback);

    println!("\n✓ All standard and custom interfaces loaded successfully!");
}
