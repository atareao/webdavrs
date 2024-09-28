# webdavrs

A WebDAV server implemented in Rust

### About the configuration file `config.yml`

It is a YAML file with the following structure:

```yaml
port: 9090
directory: /share
workers: 4
users:
  - name: user1
    hashed_password: '$2y$12$foIrZBsGcEq09HzNSKGaAOYuOcAFl23tfXyDaYdUL/wt0ug.UGlbO'
    active: true
  - name: user2
    hashed_password: '$2y$12$7ZGW1LGiV/UKLpElwJx20OzpF8dxd4deUAvcqoMM1UinuRAX6o8oS'
    active: true
```

- `port`: The port where the server will listen to.
- `directory`: The directory where the files will be stored.
- `workers`: The number of workers that will handle the requests.
- `users`: A list of users with the following fields:
  - `name`: The name of the user.
  - `hashed_password`: The hashed password of the user.
  - `active`: A boolean indicating if the user is active or not.

The password can be hashed using the following command:

```bash
htpasswd -bnBC 12 "" password | tr -d ':\n'
```

### Run the server

To run the server you can use the following command:

```bash
docker run -it -v "$HOME/sandbox:/share" -v "$PWD/config.yml:/app/config.yml" -p 9090:9090 atareao/webdavrs:v0.1.1
```

You can use `docker-compose` to run the server:

```yaml
services:
  webdavrs:
    image: atareao/webdavrs:v0.1.1
    init: true
    restart: unless-stopped
    volumes:
      - "$HOME/sandbox:/share"
      - "$PWD/config.yml:/app/config.yml"
    ports:
      - "9090:9090"
    environment:
      - RUST_LOG=debug
```

