use std::{fmt::Display, str::FromStr};
#[derive(Debug, Copy, Clone)]
pub enum Role
{
    NonPrivileged,
    User,
    Administrator
}
impl AsRef<str> for Role
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            Role::NonPrivileged => "NonPrivileged",
            Role::User => "User",
            Role::Administrator => "Administrator"
        }
    }
}


impl Display for Role
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        f.write_str(self.as_ref())
    }
}

impl From<String> for Role
{
    fn from(value: String) -> Self 
    {
        value.as_str().into()
    }
}
impl From<&String> for Role
{
    fn from(value: &String) -> Self 
    {
        value.as_str().into()
    }
}
impl From<&str> for Role
{
    fn from(value: &str) -> Self 
    {
        match value
        {
            "User" => Role::User,
            "Administrator" => Role::Administrator,
            _ => Role::NonPrivileged,
        }
    }
} 
impl FromStr for Role
{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        Ok(s.into())
    }
}
