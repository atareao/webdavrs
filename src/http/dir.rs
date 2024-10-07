use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response}
};
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use minijinja::{Environment, path_loader, context};
use tokio::fs::File;
use std::path::PathBuf;
use tracing::{debug, error};


pub static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    env
});

#[derive(Serialize, Deserialize)]
struct PathItem {
    name: String,
    path: String,
    is_dir: bool,
}

pub async fn render_directory(root: &str, path: &str) -> Option<String> {
    let file_dir = format!("{}/{}", root, path);
    debug!("Reading directory: {}", &file_dir);
    let file = File::open(&file_dir).await.unwrap();
    let main_path = PathBuf::from(&file_dir);
    let main_parent_path = main_path.parent().unwrap();
    let metadata = file.metadata().await.unwrap();
    if metadata.is_file() {
        return None;
    }

    match tokio::fs::read_dir(&file_dir).await{
        Ok(mut reader) => {
            let replace_path = format!("{}/", root);
            let replace_name = format!("{}/{}", root, path);
            let mut items = Vec::new();
            while let Ok(Some(file)) = reader.next_entry().await {
                let path = file.path();
                let path_str = path.to_str().unwrap();
                items.push(PathItem {
                    name: path_str.to_string().replace(&replace_name, "").replace("/",""),
                    path: path_str.to_string().replace(&replace_path, ""),
                    is_dir: path.is_dir(),
                });
            }
            let template = ENV.get_template("directory.html").unwrap();
            let ctx = context! {
                main_path => main_path.to_str().unwrap().replace(&format!("{root}/"), ""),
                main_parent_path => main_parent_path.to_str().unwrap().replace(&format!("{root}/"), ""),
                root => root,
                path => path,
                title => "Directory Listing",
                items => items,
            };
            template.render(&ctx).ok()
        }
        Err(e) => {
            error!("Error reading directory: {}", e);
            None
        }
    }
}
