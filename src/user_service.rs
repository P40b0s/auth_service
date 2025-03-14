// use std::sync::Arc;

// use authentification::AuthError;
// use db_service::{CountRequest, DbError, QuerySelector, Selector, SqlOperations, SqlitePool};
// use utilites::Date;
// use crate::db::{new_uid, UserSessionDbo};
// use super::DbServiceInstance;

// /// При инициализации сервиса необходимо еще запустить метод initialize  
// /// он создаст связанную таблицу в базе данных если ее там нет
// pub struct UserSessionService(Arc<SqlitePool>);
// type ServiceType = UserSessionDbo;
// impl<'a> DbServiceInstance<'a> for UserSessionService
// {
//     type DbType = ServiceType;
//     async fn new(pool: Arc<SqlitePool>) -> Self
//     {
//         let s = Self(pool);
//         s.initialize().await;
//         s 
//     }
//     fn get_db_pool(&self) -> Arc<SqlitePool>
//     {
//         Arc::clone(&self.0)
//     }
// }
// const REFRESH_KEY_LIFETIME:i64 = 60*24*5;

// impl UserSessionService
// {

//     // pub fn get_fingerprint(&self) -> Option<&String>
//     // {
//     //     self.fingerprint.as_ref()
//     // }
//     // pub fn get_ip(&self) -> &str
//     // {
//     //     &self.user_ip
//     // }
//     async fn sessions_count(&self, user_id: &str) -> Result<u32, DbError>
//     {
//         let selector = Selector::new_concat(["SELECT COUNT(*) as count FROM ", ServiceType::table_name(), " WHERE user_id = ", "\"", user_id, "\""]);
//         let count: CountRequest = ServiceType::get_one(&selector, self.get_db_pool()).await?;
//         Ok(count.count)
//     }
//     ///юзер успешно ввел логин и пароль, надо выдать ему рефреш кей
//     pub async fn new_session(&self, user_id: &str, user_ip: &str) -> Result<ServiceType, DbError>
//     {
//         let selector = Selector::new_concat([&ServiceType::full_select(), " WHERE user_id = ", "\"", user_id, "\"", " AND  user_ip = ", "\"", user_ip, "\""]);
//         let session: Result<ServiceType, DbError> = ServiceType::get_one(&selector, self.get_db_pool()).await;
//         //если сессия для таких параметров существует пробуем обновить рефреш кей и отдать обновленную сессию, иначе создаем новую сессию
//         if let Ok(s) = session
//         {
//             if let Ok(updated_session) = self.update_key(&s.id, user_ip).await
//             {
//                 return Ok(updated_session)
//             }
//         }
//         else
//         {
//             let sessions_count = self.sessions_count(&user_id).await.unwrap();
//             if sessions_count > 4
//             {
//                 let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE user_id = ", "\"", &user_id, "\""]);
//                 let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//             }
//         }
//         let updated_session = ServiceType
//         {
//             id: new_uid(user_ip.len() as u16).to_string(),
//             user_id: user_id.to_owned(),
//             logged_in: Date::now(),
//             key_expiration_time: Date::now().add_minutes(REFRESH_KEY_LIFETIME),
//             user_ip: user_ip.to_owned(),
//             fingerprint: None
//         };
//         let _ = updated_session.add_or_replace(self.get_db_pool()).await?;
//         Ok(updated_session)
//     }

//     pub async fn delete_session(&self, user_id: &str, user_ip: &str) -> Result<(), DbError>
//     {
//         let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE user_id = ", "\"", &user_id, "\"", " AND user_ip = ", "\"", &user_ip, "\"" ]);
//         let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//         Ok(())
//     }
//     pub async fn delete_all_sessions(&self, user_id: &str) -> Result<(), DbError>
//     {
//         let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE user_id = ", "\"", &user_id, "\""]);
//         let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//         Ok(())
//     }

//     ///пытаемся обновить refresh key если ок то получаем обновленную сессию, если ошибка то юзеру нужно залогиниться заново
//     pub async fn update_key(&self, key_id: &str, user_ip: &str) -> Result<ServiceType, AuthError>
//     {
//         let selector = Selector::new(ServiceType::full_select()).add_param(ServiceType::table_fields()[0], &key_id);
//         let session: Result<ServiceType, DbError> = ServiceType::get_one(&selector, self.get_db_pool()).await;
//         //ключ обновления не найден, например сессии уже были удалены по причине возможного взлома надо зайти заново
//         if session.is_err()
//         {
//             return Err(AuthError::UpdateRefreshKeyError("Ключ обновления не найден".to_owned()));
//         }
//         let session = session.unwrap();
//         //дата обновления просрочена, пусть заходит заново
//         if session.key_expiration_time.as_naive_datetime() <= Date::now().as_naive_datetime()
//         {
//             let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE id = ", "\"", key_id, "\""]);
//             let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//             return Err(AuthError::UpdateRefreshKeyError(["Время жизни ключа ", key_id, " закончилось, необходимо зайти в систему заново"].concat()));
//         }
//         if &session.user_ip != user_ip
//         {
//             let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE id = ", "\"", key_id, "\""]);
//             let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//             logger::error!("Ошибка обновления ключей, не совпадают ip: old:{} -> new:{}", &session.user_ip, user_ip);
//             return Err(AuthError::UpdateRefreshKeyError("Текущий ip адрес не совпадает с ip адресом с которого был осуществлен вход, данные текущей сессии будут сброшены".to_owned()));
//         }
//         let sessions_count = self.sessions_count(&session.user_id).await.unwrap();
//         //удаляем все сессии юзера
//         if sessions_count > 4
//         {
//             let selector = Selector::new_concat(["DELETE FROM ", ServiceType::table_name(), " WHERE user_id = ", "\"", &session.user_id, "\""]);
//             let _ = ServiceType::execute(&selector, self.get_db_pool()).await;
//         }
//         else
//         {
//             //удаляем только текущую сессию
//             let _ = session.delete(self.get_db_pool()).await;
//         }
//         let updated_session = ServiceType
//         {
//             id: new_uid(session.key_expiration_time.as_naive_datetime().and_utc().timestamp() as u16).to_string(),
//             user_id: session.user_id,
//             logged_in: session.logged_in,
//             key_expiration_time: Date::now().add_minutes(REFRESH_KEY_LIFETIME),
//             user_ip: session.user_ip,
//             fingerprint: None
//         };
//         let _ = updated_session.add_or_replace(self.get_db_pool()).await;
//         Ok(updated_session)
//     }

// }