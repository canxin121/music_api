use sea_orm::{prelude::Expr, Condition, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::{interface::server::MusicServer, models::music_aggregator};

use super::music_aggregator::MusicAggregator;

pub(crate) fn split_string(input: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = input.split("#+#").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Input string does not match the expected format."
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub(crate) async fn find_duplicate_music_agg(
    db: &DatabaseConnection,
    music_agg: &MusicAggregator,
) -> Option<String> {
    if music_agg.musics.is_empty() {
        return None;
    }
    let mut found_music_agg: Option<music_aggregator::Model> = None;
    for music in &music_agg.musics {
        match music.server {
            MusicServer::Kuwo => {
                found_music_agg = music_aggregator::Entity::find()
                    .filter(Condition::all().add(
                        Expr::col(music_aggregator::Column::KuwoMusicId).eq(music.identity.clone()),
                    ))
                    .one(db)
                    .await
                    .ok()?;
                if found_music_agg.is_some() {
                    break;
                }
            }
            MusicServer::Netease => {
                found_music_agg = music_aggregator::Entity::find()
                    .filter(
                        Condition::all().add(
                            Expr::col(music_aggregator::Column::NeteaseMusicId)
                                .eq(music.identity.clone()),
                        ),
                    )
                    .one(db)
                    .await
                    .ok()?;
                if found_music_agg.is_some() {
                    break;
                }
            }
        }
    }
    found_music_agg.and_then(|a| Some(a.identity))
}
