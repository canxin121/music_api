pub mod music_aggregator;
pub mod music_platform;
pub mod playlist;
pub mod playlist_music_junction;

#[cfg(test)]
mod orm_test {
    use music_aggregator::Entity as MusicAggregatorEntity;
    use music_aggregator::Model as MusicAggregatorModel;

    use playlist::ActiveModel as PlaylistActiveModel;
    use playlist::Entity as PlaylistEntity;

    use playlist_music_junction::ActiveModel as PlaylistMusicJunctionActiveModel;
    use playlist_music_junction::Entity as PlaylistMusicJunctionEntity;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, EntityTrait, IntoActiveModel, ModelTrait,
    };
    use sea_orm_migration::MigratorTrait;

    use crate::data::migrations::Migrator;

    use super::music_aggregator;
    use super::playlist;
    use super::playlist_music_junction;

    async fn re_init_db() -> sea_orm::DatabaseConnection {
        let db_file = "./sample_data/test.db";
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
