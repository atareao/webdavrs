use axum_auth::AuthBasic;
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

    pub fn check_auth(&self, AuthBasic((user_id, password)): AuthBasic) -> bool {
        if let Some(password) = password {
            for user in self.users.iter() {
                if user.active && user.name == user_id &&
                    verify(&password, &user.hashed_password).unwrap(){
                        return true;
                }
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
