mod connection;
mod repository;
pub use repository::{AuthorizationRepository, IAuthorizationRepository, Session};

#[cfg(test)]
mod tests
{
    use crate::{service::JwtService, role::Role};

    use super::IAuthorizationRepository;
    const USER_ID: &str = "01959414-c7e9-76c2-a84f-5aa1443b1829";
    const SESSION_ID: &str = "01959414-c7ed-7c93-b660-372d5b611c2c";
    #[tokio::test]
    async fn test_init_repo()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
    }
    #[tokio::test]
    async fn test_add_session()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.220");
        let fingerprint = "111222333444555666777888999000".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let _ = repository.create_session(&user_id, &role, refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await;
    }
    #[tokio::test]
    async fn test_add_2_session()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.221");
        let fingerprint = "02".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let _ = repository.create_session(&user_id, &role, refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await;
        //using in dyn-compatible...
        //let audience: &[Box<dyn ToString + Sync>] = &[Box::new("https://google.com"), Box::new("https://stackoverflow.com"), Box::new("https://chat.deepseek.com")];
        //let _ = repository.create_session(&user_id, Box::new(&role), refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await;
    }
    #[tokio::test]
    async fn test_add_3_session()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.222");
        let fingerprint = "03".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let _ = repository.create_session(&user_id, &role, refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await;
    }
    #[tokio::test]
    async fn test_add_4_session()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.222");
        let fingerprint = "04".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let _ = repository.create_session(&user_id, &role, refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await;
    }
    #[tokio::test]
    async fn test_replace_older_session()
    {
        let _ = logger::StructLogger::new_default();
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.002");
        let fingerprint = "02_to_05_replace".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let keys = repository.create_session(&user_id, &role, refresh_key_lifetime_days, &ip_addr, &fingerprint, Some(audience)).await.unwrap();
        logger::info!("Получены клчюи {:?}", keys);
    }
   
    #[tokio::test]
    async fn test_select_session()
    {
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let session_id: uuid::Uuid = SESSION_ID.parse().unwrap();
        let user_id: uuid::Uuid = USER_ID.parse().unwrap();
        let role = Role::NonPrivileged;
        let refresh_key_lifetime_days = 1;
        let ip_addr = String::from("182.5.202.220");
        let fingerprint = "111222333444555666777888999000".to_owned();
        let audience = &["https://google.com", "https://stackoverflow.com", "https://chat.deepseek.com"];
        let session = repository.get_session(&session_id).await.unwrap();
        assert_eq!(session.audience[1], "https://stackoverflow.com");
    }

    #[tokio::test]
    async fn test_update_keys()
    {
        let _ = logger::StructLogger::new_default();
        let jwt_service = JwtService::new();
        let repository = super::AuthorizationRepository::new(3).await.unwrap();
        let session_id: uuid::Uuid = "01959461-14cc-76f0-b3bb-ab45c01dada1".parse().unwrap();
        let keys = repository.update_session_key(&session_id, 2).await.unwrap();
        logger::info!("обновлены клчюи {:?}", keys);
    }
}