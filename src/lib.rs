mod user_service;
mod db;
mod error;
mod role;
mod auth_middleware;
mod jwt_service;
pub use jwt_service::AuthInfo;
mod state;



pub fn add(left: u64, right: u64) -> u64 
{

    left + right
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
