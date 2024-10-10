use serde::{Serialize, Deserialize};
use bcrypt::verify;

use super::{Error, User};
use tracing::debug;


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

    pub fn check_auth(&self, auth_basic: Vec<&str>) -> bool {
        if auth_basic.len() == 2 {
            let name = auth_basic[0];
            let password = auth_basic[1];
            for user in &self.users {
                debug!("{:?}", user);
                if user.active && name == user.name {
                    return verify(password, &user.hashed_password).unwrap_or(false);
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
