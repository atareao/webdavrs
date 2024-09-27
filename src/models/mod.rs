mod user;
mod config;

pub use user::User;
pub use config::Config;
pub type Error = Box<dyn std::error::Error>;

