use tokio::sync::Mutex;
use crate::{db::Repository, jwt_service::JwtService};
use super::db::IRepository;

pub struct Services<R: IRepository>
{
    pub repository: R,
}

pub struct AppState
{
    pub services: Services<Repository>
}

impl AppState
{
    pub async fn initialize() -> Result<AppState, crate::error::Error>
    {
        let jwt_service = JwtService::new();
        let repository = super::db::Repository::new(jwt_service.clone(), 3).await?;
        let services = Services
        {
            repository,
        };
        Ok(Self
        {
            services,
        })
    }

}