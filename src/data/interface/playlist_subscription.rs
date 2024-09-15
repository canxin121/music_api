use crate::server::{kuwo, netease};
use anyhow::Result;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use super::{
    music_aggregator::{Music, MusicAggregator},
    server::MusicServer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PlayListSubscription {
    #[serde(rename = "sr")]
    pub server: MusicServer,
    #[serde(rename = "se")]
    pub share: String,
}

impl PlayListSubscription {
    pub async fn fetch_musics_online(&self) -> Result<Vec<MusicAggregator>> {
        // todo: limit max need consider
        let limit = 2333;
        match self.server {
            MusicServer::Kuwo => {
                let playlist_id = kuwo::web_api::utils::find_kuwo_plylist_id_from_share_url(
                    &self.share,
                )
                .ok_or(anyhow::anyhow!(
                    "Failed to find playlist id from share url: {}",
                    self.share
                ))?;
                let kuwo_musics =
                    kuwo::web_api::playlist::get_kuwo_musics_of_music_list(&playlist_id, 1, limit)
                        .await?;
                let kuwo_musics: Vec<Music> = kuwo_musics
                    .into_iter()
                    .map(|music| music.into_music(false))
                    .collect();

                Ok(kuwo_musics
                    .into_iter()
                    .map(|music| MusicAggregator::from_music(music))
                    .collect::<Vec<MusicAggregator>>())
            }
            MusicServer::Netease => {
                let playlist_id =
                    netease::web_api::utils::find_netease_playlist_id_from_share(&self.share)
                        .ok_or(anyhow::anyhow!(
                            "Failed to find playlist id from share url: {}",
                            self.share
                        ))?;
                let models =
                    netease::web_api::playlist::get_musics_from_music_list(&playlist_id, 1, limit)
                        .await?;
                let musics: Vec<Music> = models
                    .into_iter()
                    .map(|music| music.into_music(false))
                    .collect();

                Ok(musics
                    .into_iter()
                    .map(|music| MusicAggregator::from_music(music))
                    .collect())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PlayListSubscriptionVec(pub Vec<PlayListSubscription>);
