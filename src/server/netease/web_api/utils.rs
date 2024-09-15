pub fn find_netease_playlist_id_from_share(url: &str) -> Option<String> {
    // 检查是否包含查询字符串
    if let Some(start) = url.find('?') {
        // 提取查询字符串部分
        let query_string = &url[start + 1..];

        // 解析查询字符串为键值对
        for pair in query_string.split('&') {
            let mut split = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                // 检查键是否为"id"
                if key == "id" {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}
