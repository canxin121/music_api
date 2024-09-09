pub fn format_seconds_to_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as i32;
    let remaining_seconds = seconds % 60.0;
    format!("{:02}:{:05.2}", minutes, remaining_seconds)
}
