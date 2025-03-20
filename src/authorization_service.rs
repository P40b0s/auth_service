use crate::{db::AuthorizationRepository, service::JwtService};
pub struct AuthorizationService
{
    pub repository: AuthorizationRepository,
    pub service: JwtService,
}

//update_session_key
//create_session
impl AuthorizationService
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, crate::error::Error>
    {
        let service = JwtService::new();
        let repository = super::db::AuthorizationRepository::new(max_sessions_count).await?;
        let service = AuthorizationService
        {
            repository,
            service
        };
        Ok(service)
    }
}