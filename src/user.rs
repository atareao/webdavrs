use sqlx::{sqlite::{SqlitePool, SqliteRow}, Error, query, Row};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::env;
use derive_more::Display;

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

    pub async fn exists_admin(pool: &web::Data<SqlitePool>) -> bool{
        let sql = "SELECT count(1) total FROM users where role = 'admin'";
        match query(sql)
            .map(|row: SqliteRow| -> i64 {row.get("total")})
            .fetch_one(pool.get_ref())
            .await{
                Ok(total) => total > 0,
                Err(_) => false
            }
    }

    pub fn is_admin(&self) -> bool{
        self.role == Role::Admin.to_string()
    }

    fn wrap(word: &str) -> String{
        let salt = env::var("SALT").unwrap_or("salt".to_string());
        let pepper = env::var("PEPPER").unwrap_or("pepper".to_string());
        let composition = format!("{}{}{}", salt, word, pepper);
        format!("{:x}", md5::compute(composition))
    }

    pub async fn create(pool: &web::Data<SqlitePool>, role: &str, new: &NewUser) -> Result<User, Error>{
        let hashed_password = Self::wrap(&new.password);
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
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read_and_check(auth: &BasicAuth, pool: &web::Data<SqlitePool>) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE username = $1 AND hashed_password = $2;";
        query(sql)
            .bind(auth.user_id())
            .bind(Self::wrap(auth.password().unwrap()))
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await

    }

    pub async fn read(pool: &web::Data<SqlitePool>, username: &str) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE username = $1;";
        query(sql)
            .bind(username)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn read_all(pool: &web::Data<SqlitePool>) -> Result<Vec<User>, Error>{
        let sql = "SELECT * FROM users;";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool.get_ref())
            .await
    }

    pub async fn delete(pool: &web::Data<SqlitePool>, username: &str) -> Result<User, Error>{
        let sql = "DELETE FROM users WHERE username = $1 RETURNING id,
                   username, hashed_password, role, created_at, updated_at;";
        query(sql)
            .bind(username)
            .map(Self::from_row)
            .fetch_one(pool.get_ref())
            .await
    }

    pub async fn from_auth(auth: &BasicAuth, pool: &web::Data<SqlitePool>) -> Option<User>{
        match User::read_and_check(auth, &pool).await{
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }
}

#[post("/v1/user")]
pub async fn create_user(auth: BasicAuth, pool: web::Data<SqlitePool>, new: web::Json<NewUser>) -> impl Responder{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                let role = Role::User.to_string();
                match User::create(&pool, &role, &new.into_inner()).await {
                    Ok(new_user) => HttpResponse::Created()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&new_user).unwrap()),
                    Err(_) => HttpResponse::UnprocessableEntity().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/v1/user/{username}")]
pub async fn read_user(auth: BasicAuth, pool: web::Data<SqlitePool>, path: web::Path<String>) -> impl Responder{
    let username = path.into_inner();
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::read(&pool, &username).await {
                    Ok(user) => HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&user).unwrap()),
                    Err(_) => HttpResponse::UnprocessableEntity().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/v1/user")]
pub async fn read_all_users(auth: BasicAuth, pool: web::Data<SqlitePool>) -> impl Responder{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::read_all(&pool).await {
                    Ok(users) => HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&users).unwrap()),
                    Err(_) => HttpResponse::UnprocessableEntity().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[delete("/v1/user")]
pub async fn delete_user(auth: BasicAuth, pool: web::Data<SqlitePool>, username: String) -> impl Responder{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::delete(&pool, &username).await {
                    Ok(_) => HttpResponse::Ok().finish(),
                    Err(_) => HttpResponse::NotFound().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}
