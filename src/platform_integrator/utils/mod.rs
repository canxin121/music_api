pub fn wy_extract_id(url: &str) -> Option<String> {
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

pub fn kw_extract_id(url: &str) -> Option<String> {
    // 查找路径中的playlist_detail部分
    if let Some(start) = url.find("playlist_detail/") {
        // 提取ID部分，ID在playlist_detail/后面
        let id_part = &url[start + "playlist_detail/".len()..];

        // 检查ID是否在路径中结束或者继续有查询字符串
        if let Some(end) = id_part.find('?') {
            return Some(id_part[..end].to_string());
        } else {
            return Some(id_part.to_string());
        }
    }
    None
}
