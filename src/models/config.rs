use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Serialize, Deserialize};
use bcrypt::verify;

use super::{Error, User};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    port: u16,
    directory: String,
    users: Vec<User>,
}

impl Config {

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_directory(&self) -> String {
        self.directory.clone()
    }

    pub fn check_auth(&self, creds: &BasicAuth) -> bool {
        for user in self.users.iter() {
            if user.active &&
                user.name == creds.user_id() &&
                creds.password().is_some() &&
                verify(creds.password().unwrap(), &user.hashed_password).unwrap(){
                    return true;
            }
        }
        false
    }
    pub async fn read() -> Result<Config, Error> {
        let content = tokio::fs::read_to_string("config.yml")
            .await?;
        serde_yaml::from_str(&content)
            .map_err(|e| e.into())
    }
}
