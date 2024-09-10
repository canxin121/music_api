use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KuWoQuality {
    pub(crate) level: String,
    pub(crate) bitrate: u32,
    pub(crate) format: String,
    pub(crate) size: String,
}

impl KuWoQuality {
    pub(crate) fn parse_quality(input: &str) -> Vec<KuWoQuality> {
        input
            .split(';')
            .map(|s| {
                let parts: Vec<&str> = s
                    .split(',')
                    .map(|kv| kv.split(':').nth(1).unwrap_or_default())
                    .collect();
                KuWoQuality {
                    level: parts[0].to_string(),
                    bitrate: parts[1].parse().unwrap_or_default(),
                    format: parts[2].to_string(),
                    size: parts[3].to_string(),
                }
            })
            .collect()
    }
}
pub(crate) fn process_qualities(qualities: Vec<KuWoQuality>) -> Vec<KuWoQuality> {
    let mut unique_qualities = qualities
        .into_iter()
        .filter(|q| {
            q.format != "mflac" && q.format != "zp" && q.format != "ogg" && q.format != "aac"
        })
        .fold(std::collections::HashMap::new(), |mut acc, q| {
            acc.entry((q.format.clone(), q.bitrate)).or_insert(q);
            acc
        })
        .into_values()
        .collect::<Vec<_>>();

    unique_qualities.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));

    unique_qualities
}

pub fn gen_minfo_from_formats(raw: &str) -> String {
    let mut format = String::new();
    if raw.contains("HIRFLAC") {
        format += "level:hr,bitrate:4000,format:flac,size:unknown;"
    }
    if raw.contains("ALFLAC") {
        format += "level:ff,bitrate:2000,format:flac,size:unknown;";
    }
    if raw.contains("MP3128") {
        format += "level:h,bitrate:128,format:mp3,size:unknown;";
    }
    if raw.contains("MP3H") {
        format += "level:p,bitrate:320,format:mp3,size:unknown;";
    }
    if !format.is_empty() {
        format.pop();
    }
    format
}
