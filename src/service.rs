use std::sync::Arc;
use tokio::sync::Mutex;
use jwt_authentification::{Claims, CookieService, JWT};
use crate::error::Error;

#[derive(Clone)]
pub struct JwtService
{
    jwt: Arc<Mutex<JWT>>,
    cookie: Arc<CookieService>
}
impl JwtService
{
    pub fn new() -> Self
    {
        Self
        {
            jwt: Arc::new(Mutex::new(JWT::new_in_file("key.pkcs8"))),
            cookie: Arc::new(CookieService::new_with_key("key.pkcs8"))
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
    ///validate access key, validation will not be performed on roles and audience if they are empty
    pub async fn validate<I, R, A>(&self, token: &str, roles: R, audiences: &[A]) -> Result<Claims, Error>
    where 
        I: AsRef<str>,
        R: AsRef<[I]>,
        A: ToString
    {
        let guard = self.jwt.lock().await;
        let slice: &[I] = roles.as_ref();
        let roles_str: Vec<&str> = slice.iter().map(|s| s.as_ref()).collect();
        let data = guard.validator().with_audience(audiences).with_roles(&roles_str).validate(token)?;
        Ok(data.claims)
    }
}

// fn test()
// {
//     let s = JwtService::new();
//     let v = vec![String::from("123"), String::from("321")];
//     s.validate("123321", &v, &["123123", "321321"]);
//     s.validate("123321", &["123123", "321321"], &["123123", "321321"]);
//     let arc_v = Arc::new(v);
//     s.validate("123321", &*arc_v, &["123123", "321321"]);
// }