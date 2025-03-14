use std::{clone, sync::Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utilites::http::{AUTHORIZATION, HeaderMap};
use jwt_authentification::{Claims, JWT};
use crate::error::Error;

#[derive(Clone)]
pub struct JwtService
{
    jwt: Arc<Mutex<JWT>>,
}
impl JwtService
{
    pub fn new() -> Self
    {
        Self
        {
            jwt: Arc::new(Mutex::new(JWT::new_in_file("key.pkcs8"))),
        }
    }
    ///Генерирование нового access ключа
    pub async fn gen_key<T: ToString>(&self, id: &uuid::Uuid, role: T, audience: &Vec<String>) -> String 
    {
        let mut guard = self.jwt.lock().await;
        if audience.len() > 0
        {
            guard.new_access(id).with_role(role).with_audience(audience).gen_key(5)
        }
        else 
        {
            guard.new_access(id).with_role(role).gen_key(5)
        }
        
    }
    ///проверка текущего access ключа, если валидация прошла успешно вернуться клеймы
    async fn validate(&self, token: &str) -> Result<Claims, Error>
    {
        let guard = self.jwt.lock().await;
        let data = guard.validator().validate(token)?;
        Ok(data.claims)
    }
    pub async fn get_claims(&self, headers: HeaderMap) -> Result<Claims, Error>
    {
        match headers.get(AUTHORIZATION) 
        {
            Some(value) => 
            {
                //let token_str = value.to_str().unwrap_or("")[6..].replace("Bearer ", "");
                let token_str = value.to_str().unwrap_or("")[7..].trim();
                logger::info!("Проверка токена->{}", token_str);
                let v = self.validate(&token_str).await?;
                Ok(v)
            },
            None => 
            {
                Err(Error::AuthError("Отсуствует заголовок Authorization".to_owned()))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo
{
    pub access_key: String,
    pub session_key: String
}