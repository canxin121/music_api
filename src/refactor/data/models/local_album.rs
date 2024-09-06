use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "album")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub name: String,
    #[sea_orm(nullable)]
    pub summary: Option<String>,
    #[sea_orm(nullable)]
    pub cover: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::music::Entity> for Entity {
    // The final relation is Album -> MusicAlbumJunction -> Music
    fn to() -> RelationDef {
        super::local_album_music_junction::Relation::Music.def()
    }

    fn via() -> Option<RelationDef> {
        // The original relation is MusicAlbumJunction -> Album,
        // after `rev` it becomes Album -> MusicAlbumJunction
        Some(
            super::local_album_music_junction::Relation::Album
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
