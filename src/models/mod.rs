pub mod user;
pub mod config;
pub mod response;

pub use config::Param;
pub use response::Response;
pub use user::{
    User,
    TokenClaims,
    NewUser,
    Role,
};

pub type Error = Box<dyn std::error::Error>;
