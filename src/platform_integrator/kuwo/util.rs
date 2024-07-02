pub fn decode_html_entities(input: String) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#039;", "'")
}
pub fn format_seconds_to_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as i32;
    let remaining_seconds = seconds % 60.0;
    format!("{:02}:{:05.2}", minutes, remaining_seconds)
}
