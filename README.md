# webdavrs

A WebDAV server implemented in Rust

docker run -it -v "$HOME/sandbox:/share" -v "$PWD/config.yml:/app/config.yml" -p 9090:9090 atareao/webdavrs:v0.1.1

### About the configuration file `config.yml`

It is a YAML file with the following structure:



