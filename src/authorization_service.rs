use tokio::sync::Mutex;
use crate::{db::AuthorizationRepository, jwt_service::JwtService};
use super::db::IAuthorizationRepository;

pub struct AuthorizationService<R: IAuthorizationRepository>
{
    pub repository: R,
    pub jwt_service: JwtService
}

//update_session_key
//create_session
impl AuthorizationService<AuthorizationRepository>
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, crate::error::Error>
    {
        let jwt_service = JwtService::new();
       
        let repository = super::db::AuthorizationRepository::new(max_sessions_count).await?;
        let service = AuthorizationService
        {
            repository,
            jwt_service
        };
        Ok(service)
    }
}