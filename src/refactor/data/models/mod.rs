pub mod music_aggregator;
pub mod music_platform;
pub mod playlist;
pub mod playlist_music_junction;

pub use music_aggregator::ActiveModel as MusicAggregatorActiveModel;
pub use music_aggregator::Entity as MusicAggregatorEntity;
pub use music_aggregator::Model as MusicAggregatorModel;

pub use playlist::ActiveModel as PlaylistActiveModel;
pub use playlist::Entity as PlaylistEntity;
pub use playlist::Model as PlaylistModel;

pub use playlist_music_junction::ActiveModel as PlaylistMusicJunctionActiveModel;
pub use playlist_music_junction::Entity as PlaylistMusicJunctionEntity;
pub use playlist_music_junction::Model as PlaylistMusicJunctionModel;

#[cfg(test)]
mod orm_test {
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, EntityTrait, IntoActiveModel, ModelTrait,
    };
    use sea_orm_migration::MigratorTrait;

    use crate::refactor::data::{
        migrations::Migrator,
        models::{
            MusicAggregatorEntity, MusicAggregatorModel, PlaylistActiveModel, PlaylistEntity,
            PlaylistMusicJunctionActiveModel, PlaylistMusicJunctionEntity,
        },
    };

    async fn re_init_db() -> sea_orm::DatabaseConnection {
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        let db = sea_orm::Database::connect("sqlite://".to_owned() + db_file)
            .await
            .unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }
    async fn insert_some_data(db: &sea_orm::DatabaseConnection) {
        let album = PlaylistActiveModel::new(
            "Album".to_owned(),
            Some("Summary".to_owned()),
            Some("Cover".to_owned()),
            1,
        );
        let album_active = album.save(db).await.unwrap();

        let music1_model = MusicAggregatorModel::default();
        println!("{:?}", music1_model);

        let mut music1_active = music1_model.into_active_model();
        music1_active.identity = NotSet;
        let music1_active = music1_active.save(db).await.unwrap();

        let junction_active = PlaylistMusicJunctionActiveModel::new(
            album_active.id.unwrap(),
            music1_active.identity.unwrap(),
            1,
        );
        PlaylistMusicJunctionEntity::insert(junction_active)
            .exec(db)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn test_db() {
        re_init_db().await;
    }

    #[tokio::test]
    async fn test_insert_all() {
        let db = re_init_db().await;
        insert_some_data(&db).await;
    }

    #[tokio::test]
    async fn test_query() {
        let db = re_init_db().await;
        insert_some_data(&db).await;

        let playlists = PlaylistEntity::find().all(&db).await.unwrap();
        println!("{:?}", playlists);

        let playlist1 = playlists.first().unwrap();

        let musics = playlist1
            .find_related(MusicAggregatorEntity)
            .all(&db)
            .await
            .unwrap();
        println!("{:?}", musics);
    }
}
