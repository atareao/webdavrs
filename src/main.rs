mod http;
mod models;

use sqlx::{
    sqlite::{
        SqlitePool,
        SqlitePoolOptions
    },
    migrate::{
        Migrator,
        MigrateDatabase
    }
};
use std::{
    env::{
        self,
        var
    },
    path::Path,
    process
};
use dav_server::{
    fakels::FakeLs,
    localfs::LocalFs,
    DavHandler,
    actix::{
        DavRequest,
        DavResponse
    }
};
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt
};
use crate::{
    list::Lister,
    user::{
        User,
        NewUser,
        Role,
        create_user,
        read_user,
        read_all_users,
        delete_user
    },
};


pub async fn dav_handler(auth: BasicAuth, req: DavRequest, davhandler: Data<DavHandler>, folder: Data<String>, pool: Data<SqlitePool>) -> DavResponse{
    if User::read_and_check(&auth, &pool).await.is_err(){
        let res = http::Response::builder()
            .status(401)
            .body(dav_server::body::Body::empty())
            .unwrap();
        DavResponse(res)
    }else{
        info!("{}", auth.user_id());
        if req.prefix().is_some(){
            let res = http::Response::builder()
                .body(dav_server::body::Body::empty())
                .unwrap();
            DavResponse(res)
        }else{
            info!("{:?}", "aqui");
            info!("{:?}", req.prefix());
            info!("{:?}", req.request.method());
            if req.request.method() == "GET" && req.request.uri().to_string().ends_with("/"){
                let maindir = folder.into_inner().to_string();
                let subdir = req.request.uri().to_string();
                info!("maindir: {}", &maindir);
                info!("subdir: {}", &subdir);
                let lister = Lister::new("Ejemplo", &maindir, &subdir);
                println!("{}", lister.generate().await);
                let content = lister.generate().await;
                let body: dav_server::body::Body = content.into();
                let res = http::Response::builder()
                    .body(body).unwrap();
                    DavResponse(res)
            }else{
                davhandler.handle(req.request).await.into()
            }
        }
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");
    let db_url = var("DB_URL").unwrap_or("wevdavrs.db".to_string());
    info!("DB url: {db_url}");

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap(){
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let migrations = if var("RUST_ENV") == Ok("production".to_string()){
        std::env::current_exe().unwrap().parent().unwrap().join("migrations")
    }else{
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&crate_dir).join("migrations")
    };
    info!("{}", &migrations.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Pool failed");

    info!("Start migrations");
    Migrator::new(migrations)
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    info!("Migrations done");

    let data_pool = Data::new(pool.clone());
    if !User::exists_admin(&data_pool).await{
        let role = Role::Admin.to_string();
        let new = NewUser {username, password};
        match User::create(&data_pool, &role, &new).await{
            Ok(_) => {
                info!("Created admin user");
            },
            Err(_) => {
                error!("Can not create admin user");
                process::exit(1);
            }
        };
    }else{
        info!("The admin user exists");
    }

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(&folder, false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    tracing::info!("🚀 Server started successfully");
    http::serve(&pool)
        .await
        .unwrap();


    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(pool.clone()))
            .app_data(basic::Config::default().realm("Restricted area"))
            .app_data(Data::new(dav_server.clone()))
            .app_data(Data::new(folder.clone()))
            .service(create_user)
            .service(read_user)
            .service(read_all_users)
            .service(delete_user)
            .service(web::resource("/{tail:.*}").to(dav_handler))
    })
    .workers(4)
    .bind(format!("0.0.0.0:{}", &port))
    .unwrap()
    .run()
    .await
}
