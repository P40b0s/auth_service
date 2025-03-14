use tokio::sync::Mutex;
use crate::{db::Repository, jwt_service::JwtService};

use super::db::IRepository;
#[derive(Clone)]
pub struct Settings
{
    //pub processing_documets_types : Vec<PublicationDocumentTypeDbo>
}
pub struct Services<R: IRepository>
{
    /// Сервис базы данных предоставляет только пул соединений
    pub repository: R,
    ///JWT сервис предоставляет методы для валидации ключа доступа и создания нового ключа доступа
    pub jwt_service: JwtService
}

pub struct AppState
{
    pub settings: Mutex<Settings>,
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
            jwt_service,
        };
        Ok(Self
        {
            services,
            settings: Mutex::new(Settings
            {
                
            }),
        })
    }
    pub async fn get_settings(&self) -> Settings
    {
        let guard = self.settings.lock().await;
        guard.clone()
    }
}