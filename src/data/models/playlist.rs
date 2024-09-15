use crate::{
    data::interface::music_aggregator::{Music, MusicAggregator},
    data::interface::server::MusicServer,
    server::{kuwo, netease},
};
use anyhow::Result;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, FromJsonQueryResult, Set};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub order: i64,
    pub name: String,
    #[sea_orm(nullable)]
    pub summary: Option<String>,
    #[sea_orm(nullable)]
    pub cover: Option<String>,
    pub subscriptions: Option<PlayListSubscriptionVec>,
}

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
impl ActiveModel {
    pub fn new(name: String, summary: Option<String>, cover: Option<String>, order: i64) -> Self {
        Self {
            id: NotSet,
            name: Set(name),
            summary: Set(summary),
            cover: Set(cover),
            subscriptions: (Default::default()),
            order: Set(order),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Id)
                    .to(super::playlist_music_junction::Column::PlaylistId)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .on_update(sea_query::ForeignKeyAction::Cascade)
                    .into()
            }
        }
    }
}

impl Related<super::music_aggregator::Entity> for Entity {
    // The final relation is Album -> MusicAlbumJunction -> Music
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::MusicAggregator.def()
    }

    fn via() -> Option<RelationDef> {
        // The original relation is MusicAlbumJunction -> Album,
        // after `rev` it becomes Album -> MusicAlbumJunction
        Some(
            super::playlist_music_junction::Relation::Playlist
                .def()
                .rev(),
        )
    }
}

impl Related<super::playlist_music_junction::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::Playlist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
