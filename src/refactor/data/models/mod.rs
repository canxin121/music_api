pub mod music;
pub mod music_platform;
pub mod local_album;
pub mod local_album_music_junction;

pub use music::Model as MusicModel;
pub use music::ActiveModel as MusicActiveModel;
pub use music::Entity as MusicEntity;

pub use local_album::Model as LocalAlbumModel;
pub use local_album::ActiveModel as LocalAlbumActiveModel;
pub use local_album::Entity as LocalAlbumEntity;


#[cfg(test)]
mod orm_test{
    async fn init_db()->sea_orm::DatabaseConnection{
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists(){
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();
        
        let db = sea_orm::Database::connect("sqlite://".to_owned() + db_file).await.unwrap();
        db
    }
    #[tokio::test]
    async fn test_db(){
        init_db().await;
    }
    
}