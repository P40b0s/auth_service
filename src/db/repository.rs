use std::sync::Arc;
use sqlx::{query::Query, sqlite::{SqliteArguments, SqliteRow}, FromRow, Row, Sqlite, SqlitePool};
use utilites::Date;
use crate::error::Error;

#[derive(Clone)]
pub struct AuthorizationRepository
{
    connection: Arc<SqlitePool>,
    max_sessions_count: u8
}

impl AuthorizationRepository
{
    pub async fn new(max_sessions_count: u8) -> Result<Self, Error>
    {
        let pool = Arc::new(super::connection::new_connection("sessions").await?);
        let r1 = sqlx::query(create_table_sql()).execute(&*pool).await;
        if r1.is_err()
        {
            logger::error!("{}", r1.as_ref().err().unwrap());
            let _ = r1?;
        };
        Ok(Self
        {
            connection: pool,
            max_sessions_count
        })
    }
}
pub trait IAuthorizationRepository
{
    fn create_session<R: ToString + Copy + Send, T: ToString + Sync>(&self, id: &uuid::Uuid, role: R, refresh_key_lifetime_days: i64, ip_addr: &str, fingerprint: &str, audience: Option<&[T]>) -> impl std::future::Future<Output = Result<uuid::Uuid, Error>> + Send;
    fn get_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<UserSessionDbo, Error>> + Send;
    fn insert_or_replace_session(&self, session: UserSessionDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn sessions_count(&self, id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u32, Error>> + Send;
    fn delete_all_sessions(&self, id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u64, Error>> + Send;
    fn delete_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn update_session_key(&self, session_id: &uuid::Uuid, refresh_key_lifetime_days: i64) -> impl std::future::Future<Output = Result<(), Error>>;
}

fn create_table_sql<'a>() -> &'a str
{
    "BEGIN;
    CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    logged_in TEXT NOT NULL,
    audience TEXT NOT NULL DEFAULT('[]'),
    role TEXT NOT NULL,
    key_expiration_time TEXT NOT NULL,
    ip_addr TEXT NOT NULL,
    fingerprint TEXT,
    PRIMARY KEY(id, session_id)
    );
    CREATE INDEX IF NOT EXISTS 'session_idx' ON sessions (id, session_id, role);
    COMMIT;"
}

enum UserSessionTable
{
    Id,
    SessionId,
    LoggedIn,
    Audience,
    Role,
    KeyExpirationTime,
    IpAddr,
    Fingerprint
}

impl UserSessionTable
{
    pub fn get_all() -> String
    {
        [
            UserSessionTable::Id.as_ref(), ",", 
            UserSessionTable::SessionId.as_ref(), ",", 
            UserSessionTable::LoggedIn.as_ref(), ",", 
            UserSessionTable::Audience.as_ref(), ",", 
            UserSessionTable::Role.as_ref(), ",", 
            UserSessionTable::KeyExpirationTime.as_ref(), ",", 
            UserSessionTable::IpAddr.as_ref(), ",", 
            UserSessionTable::Fingerprint.as_ref(), 
        ].concat()
    }
}

impl AsRef<str> for UserSessionTable
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            UserSessionTable::Id => "id",
            UserSessionTable::SessionId => "session_id",
            UserSessionTable::LoggedIn => "logged_in",
            UserSessionTable::Audience => "audience",
            UserSessionTable::Role => "role",
            UserSessionTable::KeyExpirationTime => "key_expiration_time",
            UserSessionTable::IpAddr => "ip_addr",
            UserSessionTable::Fingerprint => "fingerprint"
        }
    }
}

#[derive(Debug)]
pub struct UserSessionDbo 
{
    pub id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub logged_in: Date,
    pub audience: Vec<String>,
    pub role: String,
    pub key_expiration_time: Date,
    pub ip_addr: String,
    pub fingerprint: String
}

impl UserSessionDbo
{
    pub fn bind_all<'a>(&'a self, sql: &'a str) -> Query<'a, Sqlite, SqliteArguments<'a>>
    {
        sqlx::query(&sql)
        .bind(self.id.to_string())
        .bind(self.session_id.to_string())
        .bind(self.logged_in.to_string())
        .bind(serde_json::to_string(&self.audience).unwrap())
        .bind(self.role.as_str())
        .bind(self.key_expiration_time.to_string())
        .bind(&self.ip_addr)
        .bind(&self.fingerprint)
    }
}

impl FromRow<'_, SqliteRow> for UserSessionDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let id: &str =  row.try_get(UserSessionTable::Id.as_ref())?;
        let session_id: &str =  row.try_get(UserSessionTable::SessionId.as_ref())?;
        let logged_in: &str =  row.try_get(UserSessionTable::LoggedIn.as_ref())?;
        let audience: &str = row.try_get(UserSessionTable::Audience.as_ref())?;
        let role: &str = row.try_get(UserSessionTable::Role.as_ref())?;
        let key_expiration_time: &str = row.try_get(UserSessionTable::KeyExpirationTime.as_ref())?;
        let ip_addr: &str = row.try_get(UserSessionTable::IpAddr.as_ref())?;
        let fingerprint: String = row.try_get(UserSessionTable::Fingerprint.as_ref())?;

        let obj = UserSessionDbo   
        {
            id: id.parse().unwrap(),
            session_id: session_id.parse().unwrap(),
            logged_in: Date::parse(logged_in).unwrap(),
            audience: serde_json::from_str(audience).unwrap(),
            role: role.to_owned(),
            key_expiration_time: Date::parse(key_expiration_time).unwrap(),
            ip_addr: ip_addr.to_owned(),
            fingerprint
        };
        Ok(obj)
    }
}

impl IAuthorizationRepository for AuthorizationRepository
{
    fn create_session<R: ToString + Copy + Send, T: ToString + Sync>(&self, id: &uuid::Uuid, role: R, refresh_key_lifetime_days: i64, ip_addr: &str, fingerprint: &str, audience: Option<&[T]>) -> impl std::future::Future<Output = Result<uuid::Uuid, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let audience = audience.unwrap_or(&Vec::new()).iter().map(|a| a.to_string()).collect();
            let sql = ["SELECT ", &UserSessionTable::get_all(), " FROM sessions WHERE ", UserSessionTable::Id.as_ref(), " = $1 ORDER BY ", UserSessionTable::LoggedIn.as_ref()].concat();
            let mut current_sessions = sqlx::query_as::<_, UserSessionDbo>(&sql)
            .bind(id.to_string())
            .fetch_all(&*connection).await?;
            //sessions for current user not exists
            if current_sessions.is_empty()
            {
                let session = new_session(id, role, refresh_key_lifetime_days, ip_addr, fingerprint, audience);
                let session_id = session.session_id.clone();
                let _ = self.insert_or_replace_session(session).await?;
                Ok(session_id)
            }
            //sessions count bigger than 3, replace older session with updated session
            else if current_sessions.len() > self.max_sessions_count as usize
            {
                let old_session = current_sessions.swap_remove(0);
                //if fingerprint equalis
                if let Some(mut session) = current_sessions.into_iter().find(|f|f.fingerprint == fingerprint)
                {
                    session.ip_addr = ip_addr.to_owned();
                    session.logged_in = Date::now();
                    session.role = role.to_string();
                    session.key_expiration_time = Date::now().add_minutes(refresh_key_lifetime_days*60*24);
                    session.audience = audience;
                    let session_id = session.session_id.clone();
                    let _ = self.insert_or_replace_session(session).await?;
                    Ok(session_id)
                }
                else 
                {
                    self.delete_session(&old_session.session_id).await?;
                    let session = new_session(id, role, refresh_key_lifetime_days, ip_addr, fingerprint, audience);
                    let session_id = session.session_id.clone();
                    let _ = self.insert_or_replace_session(session).await?;
                    logger::warn!("Превышено максимальное количество одновременных сессий `{}` сессия `{}` заменена на {}", self.max_sessions_count, &old_session.session_id.to_string(), &session_id);
                    Ok(session_id)
                }
            }
            else 
            {
                //sessions with equalis fingerprint is found, update session and return new keys
                if let Some(mut session) = current_sessions.into_iter().find(|f|f.fingerprint == fingerprint)
                {
                    session.ip_addr = ip_addr.to_owned();
                    session.logged_in = Date::now();
                    session.role = role.to_string();
                    session.key_expiration_time = Date::now().add_minutes(refresh_key_lifetime_days*60*24);
                    session.audience = audience;
                    let session_id = session.session_id.clone();
                    let _ = self.insert_or_replace_session(session).await?;
                    Ok(session_id)
                }
                //add new session for this user
                else 
                {
                    let session = new_session(id, role, refresh_key_lifetime_days, ip_addr, fingerprint, audience);
                    let session_id = session.session_id.clone();
                    let _ = self.insert_or_replace_session(session).await?;
                    Ok(session_id)
                }
            }
        })
    }
    //update current session lifetime
    fn update_session_key(&self, session_id: &uuid::Uuid, refresh_key_lifetime_days: i64) -> impl std::future::Future<Output = Result<(), Error>>
    {
        Box::pin(async move 
        {
            let mut session = self.get_session(session_id).await?;
            if session.key_expiration_time > Date::now()
            {
                session.key_expiration_time = Date::now().add_minutes(refresh_key_lifetime_days*60*24);
                self.insert_or_replace_session(session).await?;
                Ok(())
            }
            else 
            {
                Err(Error::SessionExpired)
            }
        })
    }
    fn get_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<UserSessionDbo, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["SELECT ", &UserSessionTable::get_all(), " FROM sessions WHERE ", UserSessionTable::SessionId.as_ref(), " = $1"].concat();
            let  current_session = sqlx::query_as::<_, UserSessionDbo>(&sql)
            .bind(session_id.to_string())
            .fetch_one(&*connection).await;
            if let Ok(session) = current_session
            {
                Ok(session)
            }
            else 
            {
                Err(Error::SessionNotFound)
            }
            
        })
        
    }
    fn insert_or_replace_session(&self, session: UserSessionDbo) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["INSERT OR REPLACE INTO sessions (", &UserSessionTable::get_all(), ") VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"].concat();
            let _ = session.bind_all(&sql)
            .execute(&*connection).await?;
            Ok(())
        })
    }

    fn sessions_count(&self, id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u32, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["SELECT COUNT(*) FROM sessions WHERE ", UserSessionTable::Id.as_ref(), " = $1"].concat();
            let count: u32 = sqlx::query_scalar(&sql)
            .bind(id.to_string())
            .fetch_one(&*connection).await?;
            Ok(count)
        })
    }
    fn delete_all_sessions(&self, id: &uuid::Uuid) -> impl std::future::Future<Output = Result<u64, Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["DELETE FROM sessions WHERE ", UserSessionTable::Id.as_ref(), " = $1"].concat();
            let count = sqlx::query(&sql)
            .bind(id.to_string())
            .execute(&*connection).await?;
            let count = count.rows_affected();
            logger::info!("Для `{}` удалено `{}` сессий", id.to_string(), count);
            Ok(count)
        })
    }
    fn delete_session(&self, session_id: &uuid::Uuid) -> impl std::future::Future<Output = Result<(), Error>> + Send
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = ["DELETE FROM sessions WHERE ", UserSessionTable::SessionId.as_ref(), " = $1"].concat();
            let _ = sqlx::query(&sql)
            .bind(session_id.to_string())
            .execute(&*connection).await?;
            logger::info!("Удалена сессия `{}`", session_id.to_string());
            Ok(())
        })
    }
}

fn new_session<R: ToString>(id: &uuid::Uuid, role: R, refresh_key_lifetime_days: i64, ip_addr: &str, fingerprint: &str, audience: Vec<String>) -> UserSessionDbo
{
    
    UserSessionDbo
    {
        id: id.clone(),
        session_id: uuid::Uuid::now_v7(),
        logged_in: Date::now(),
        audience,
        role: role.to_string(),
        key_expiration_time: Date::now().add_minutes(refresh_key_lifetime_days*60*24),
        ip_addr: ip_addr.to_owned(),
        fingerprint: fingerprint.to_owned()
    }
}