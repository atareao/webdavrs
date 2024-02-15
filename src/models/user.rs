use axum_auth::AuthBasic;
use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use md5;
use derive_more::Display;

use crate::models::config::Param;

#[derive(Debug, Display, PartialEq)]
pub enum Role{
    #[display(fmt = "admin")]
    Admin,
    #[display(fmt = "user")]
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub hashed_password: String,
    pub active: bool,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token<'a> {
    pub token: &'a str,
}

impl User{
    fn from_row(row: SqliteRow) -> User{
        User {
            id: row.get("id"),
            username: row.get("username"),
            hashed_password: row.get("hashed_password"),
            role: row.get("role"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn exists_admin(pool: &SqlitePool) -> bool{
        let sql = "SELECT count(1) total FROM users where role = 'admin'";
        match query(sql)
            .map(|row: SqliteRow| -> i64 {row.get("total")})
            .fetch_one(pool)
            .await{
                Ok(total) => total > 0,
                Err(_) => false
            }
    }

    pub fn is_admin(&self) -> bool{
        self.role == Role::Admin.to_string()
    }

    async fn wrap(pool: &SqlitePool, word: &str) -> Result<String, Error>{
        let salt = Param::get_salt(pool).await?;
        let pepper = Param::get_pepper(pool).await?;
        let composition = format!("{}{}{}", salt, word, pepper);
        Ok(format!("{:x}", md5::compute(composition)))
    }

    pub async fn create(pool: &SqlitePool, role: &str, new: &NewUser) -> Result<User, Error>{
        let hashed_password = Self::wrap(pool, &new.password);
        let created_at = Utc::now();
        let updated_at = Utc::now();
        let sql = "INSERT INTO users (username, hashed_password, role,
            active, created_at, updated_at) VALUES($1, $2, $3, $4, $5, $6) 
            RETURNING *";
        query(sql)
            .bind(&new.username)
            .bind(&hashed_password)
            .bind(role)
            .bind(true)
            .bind(created_at)
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn update(pool: &SqlitePool, user: User) -> Result<User, Error>{
        let updated_at = Utc::now();
        let sql = "UPDATE users (hashed_password, role, active, updated_at)
            VALUES($2, $3, $4, $5) WHERE username = $1 RETURNING *";
        query(sql)
            .bind(&user.username)
            .bind(&user.hashed_password)
            .bind(&user.role)
            .bind(&user.active)
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read_and_check(AuthBasic((username, password)): AuthBasic, pool: &SqlitePool) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE username = $1 AND hashed_password = $2;";
        let hashed_password = Self::wrap(pool,password).await?;
        query(sql)
            .bind(username)
            .bind(hashed_password)
            .map(Self::from_row)
            .fetch_one(pool)
            .await

    }

    pub async fn read(pool: &SqlitePool, username: &str) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE username = $1;";
        query(sql)
            .bind(username)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<User>, Error>{
        let sql = "SELECT * FROM users;";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
    }

    pub async fn delete(pool: &SqlitePool, username: &str) -> Result<User, Error>{
        let sql = "DELETE FROM users WHERE username = $1 RETURNING *;";
        query(sql)
            .bind(username)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn from_auth(auth: AuthBasic, pool: &SqlitePool) -> Option<User>{
        match User::read_and_check(auth, &pool).await{
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }
}
