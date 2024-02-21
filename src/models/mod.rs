pub mod user;
pub mod config;

pub use config::Param;
pub use user::{
    User,
    NewUser,
    Role,
};

pub type Error = Box<dyn std::error::Error>;
