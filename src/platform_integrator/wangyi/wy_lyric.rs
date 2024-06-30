use super::request::eapi_request;
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
struct LyricResult {
    lrc: Lrc,
}
#[derive(Deserialize)]
struct Lrc {
    #[allow(unused)]
    version: u32,
    lyric: String,
}

#[derive(Deserialize)]
struct ItemOfC {
    tx: String,
}
#[derive(Deserialize)]
struct MapPart {
    t: u32,
    c: Vec<ItemOfC>,
}

impl MapPart {
    fn parse(self) -> String {
        fn format_lrc_timestamp(milliseconds: u32) -> String {
            // 将毫秒转换为分钟、秒和百分之一秒
            let minutes = milliseconds / 60000;
            let seconds = (milliseconds % 60000) / 1000;
            let hundredths = (milliseconds % 1000) / 10;

            // 使用格式化字符串构造时间戳
            format!("[{:02}:{:02}.{:02}]", minutes, seconds, hundredths)
        }
        let mut result = String::new();
        result += &format_lrc_timestamp(self.t);
        result += &self
            .c
            .into_iter()
            .map(|c| c.tx)
            .collect::<Vec<String>>()
            .join(" ");
        result
    }
}
impl Lrc {
    fn parse(self) -> String {
        let mut lrc = String::new();
        let data = self.lyric.replace(r#"\""#, r#"""#);
        let parts = data.split("\n");
        for part in parts {
            if part.starts_with("[") {
                lrc += &part;
                lrc += "\n";
            } else if part.starts_with("{") {
                if let Ok(map_part) = serde_json::from_str::<MapPart>(part) {
                    lrc += &map_part.parse();
                    lrc += "\n";
                }
            }
        }
        lrc
    }
}

pub async fn get_lyric(music_id: &str) -> Result<String, anyhow::Error> {
    let data = &json!({
      "id": music_id,
      "cp": false,
      "tv": 0,
      "lv": 0,
      "rv": 0,
      "kv": 0,
      "yv": 0,
      "ytv": 0,
      "yrv": 0,
    })
    .to_string();
    let resp = eapi_request(r#"/api/song/lyric/v1"#, &data).await?;
    let lyric_result = serde_json::from_str::<LyricResult>(&resp)?;
    Ok(lyric_result.lrc.parse())
}

#[tokio::test]
async fn test_get_lyric() {
    let music_id = "522352195";
    let result = get_lyric(music_id).await.unwrap();
    println!("{}", result);
    std::fs::write("lyric.lrc", result).expect("Failed to write result to file");
}
