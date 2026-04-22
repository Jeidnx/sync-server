# LibreTube Sync Server
Server to synchronize streaming service data (e.g. subscriptions, playlists) between devices, built for LibreTube.

## Running
It's recommended to run the app with Docker.

There are multiple prebuilt Docker images, built for ARM64 and x86:
- `latest-postgres`: uses PostgresQL as database backend
- `latest-sqlite`: uses SQLite as database backend

For reference, please see the example `docker-compose` files at [docker-compose.yml](./docker-compose.yml) and [docker-compose.postgres.yml](./docker-compose.postgres.yml).

After you chose the correct `docker-compose.yml` for your use case, just run `docker compose up`.

## API Documentation
- Start the app, e.g. with `cargo run`.
- The documentation can now be found at `http://localhost:8080/docs`.

### Authentication
After registering or logging in, you receive a `jwt` as response.

This `jwt` must be passed either as `Authorization` cookie or header for authenticated requests, e.g. for creating subscriptions.
For example:
- Header: `Authorization: abcdefghijklmnopqrtuvwxyz`
- Cookie: `Authorization=abcdefghijklmnopqrtuvwxyz`

## Developing
### Adding New Database Objects or Altering Tables
+ Create a new migration with `diesel migration generate <migration_name>` 
+ Edit the `up.sql` and `down.sql` files in `migrations/..._<migration_name>`. E.g., add a `SQL CREATE TABLE` statement or alter an existing table by adding a new field.
+ Manually create Rust structs for it in `src/models.rs`.

For more information, see <https://diesel.rs/guides/getting-started>.
