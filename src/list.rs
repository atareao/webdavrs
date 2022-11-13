use serde::{Serialize, Deserialize};
use std::path::Path;
use tokio::fs;
use tera::{Tera, Context};


const HTML: &str = r#"<!DOCTYPE html>

<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>{{title}}</title>

    <link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/octoshrimpy/microcss@main/micro.css">
    <style type="text/css">
        main {
            max-width: 70ch;
            padding: 2ch;
            margin: auto;
        }
        ul{list-style-type: none;}
        a{text-decoration: none}
    </style>

  </head>

  <main>
    <h1>{{title}}</h1>

    <ul>
    {% for file in files %}
        <li><a href="{{path}}{{file.file}}">{{file.emoji}} {{file.file}}</a></li>
    {% endfor %}
    </ul>

  </main>
</html>"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct Item{
    file: String,
    emoji: String,
}

impl Item{
    fn new(file: &str, emoji: &str) -> Item{
        Self{
            file: file.to_string(),
            emoji: emoji.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Lister<'a>{
    title: &'a str,
    maindir: &'a str,
    subdir: &'a str,
}

impl<'a> Lister<'a>{
    pub fn new(title: &'a str, maindir: &'a str, subdir: &'a str) -> Lister<'a>{
        Self{
            title,
            maindir,
            subdir,
        }
    }

    pub async fn generate(&self) -> String{
        let mut files = Vec::new();
        let path = format!("{}{}", self.maindir, self.subdir);
        let mut entries = fs::read_dir(Path::new(&path)).await.unwrap();
        if self.subdir != "/"{
            files.push(Item::new("..", "📁"));
        }
        while let Some(entry) = entries.next_entry().await.unwrap() {
            // Here, `entry` is a `DirEntry`.
            let filename = entry.file_name().to_str().unwrap().to_string();
            let item = if entry.file_type().await.unwrap().is_dir(){
                Item::new(&format!("{}/", filename), "📁")
            }else{
                Item::new(&format!("{}", filename), "📄")
            };
            files.push(item);
        }
        let mut template = Tera::default();
        template.add_raw_template("template.html", HTML).unwrap();
        let mut context = Context::new();
        context.insert("title", self.title);
        context.insert("path", self.subdir);
        context.insert("files", &files);
        
        return template.render("template.html", &context).unwrap();
    }

}
