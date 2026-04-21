# LibreTube Sync Server

## Adding New Database Objects or Altering Tables
+ Create a new migration with `diesel migration generate <migration_name>` 
+ Edit the `up.sql` and `down.sql` files in `migrations/..._<migration_name>`. E.g., add a `SQL CREATE TABLE` statement or alter an existing table by adding a new field.
+ Manually create Rust structs for it in `src/models.rs`.

For more information, see <https://diesel.rs/guides/getting-started>.
