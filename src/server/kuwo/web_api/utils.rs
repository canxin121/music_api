use crate::{data::interface::quality::Quality, CLIENT};

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

pub fn parse_qualities_minfo(input: &str) -> Vec<Quality> {
    let mut qualities: Vec<Quality> = input
        .split(';')
        .map(|s| {
            let parts: Vec<&str> = s
                .split(',')
                .map(|kv| kv.split(':').nth(1).unwrap_or_default())
                .collect();
            Quality {
                summary: format!("{}k{}", parts[1].to_string(), parts[2].to_string()),
                bitrate: Some(parts[1].to_string()),
                format: Some(parts[2].to_string()),
                size: Some(parts[3].to_string()),
            }
        })
        .filter(|q| {
            if let Some(format) = &q.format {
                format != "mflac" && format != "zp" && format != "ogg" && format != "aac"
            } else {
                false
            }
        })
        .collect();
    qualities.sort_by(|a, b| {
        let a_bitrate = a.bitrate.as_deref().unwrap_or("");
        let b_bitrate = b.bitrate.as_deref().unwrap_or("");
        b_bitrate
            .parse::<u32>()
            .unwrap_or_default()
            .cmp(&a_bitrate.parse::<u32>().unwrap_or_default())
    });
    qualities
}

pub fn parse_qualities_formats(formats: &str) -> Vec<Quality> {
    let mut minfo = String::new();
    if formats.contains("HIRFLAC") {
        minfo += "level:hr,bitrate:4000,format:flac,size:unknown;"
    }
    if formats.contains("ALFLAC") {
        minfo += "level:ff,bitrate:2000,format:flac,size:unknown;";
    }
    if formats.contains("MP3128") {
        minfo += "level:h,bitrate:128,format:mp3,size:unknown;";
    }
    if formats.contains("MP3H") {
        minfo += "level:p,bitrate:320,format:mp3,size:unknown;";
    }
    if !minfo.is_empty() {
        minfo.pop();
    }

    parse_qualities_minfo(&minfo)
}

pub async fn get_music_rid_pic(music_rid: &str) -> Result<String, anyhow::Error> {
    let url = format!(
        "http://artistpicserver.kuwo.cn/pic.web?corp=kuwo&type=rid_pic&pictype=500&size=500&rid={}",
        music_rid.replace("MUSIC_", "")
    );
    let resp = CLIENT.get(&url).send().await?;
    let text = resp.text().await?;
    Ok(text)
}

pub fn find_kuwo_plylist_id_from_share_url(url: &str) -> Option<String> {
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
