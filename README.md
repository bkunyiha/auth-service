## Setup & Building
```bash
cargo install cargo-watch
cd app-service
cargo build
cd ..
cd auth-service
cargo build
cd ..
```

## Run servers locally (Manually)

#### Setup Migrations
cd auth-service && sqlx database create && sqlx migrate run

#### App service
```bash
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
```

visit http://localhost:8000

#### Auth service
```bash
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run
```

visit http://localhost:3000

## Run servers locally (Docker)
```bash
./docker.sh
```

visit http://localhost:8000 and http://localhost:3000