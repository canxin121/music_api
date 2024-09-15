pub(crate) fn is_artist_equal(one: &str, two: &str) -> bool {
    let one = one.split('&').collect::<Vec<&str>>();
    let two = two.split('&').collect::<Vec<&str>>();
    if one.len() != two.len() {
        return false;
    }
    for i in 0..one.len() {
        if !two.contains(&one[i]) {
            return false;
        }
    }
    true
}

pub(crate) fn split_string(input: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = input.split("#+#").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Input string does not match the expected format."
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
