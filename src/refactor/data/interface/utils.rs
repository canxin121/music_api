pub fn is_artist_equal(one: &str, two: &str) -> bool {
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
