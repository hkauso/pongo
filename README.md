# 🦧 Pongo

Pongo is a simple and intelligent [Dorsal](https://github.com/stellularorg/dorsal)-based admin interface for database management.

## Examples

Example usage (serving just the admin panel) can be found in the `examples/` directory [here](https://github.com/hkauso/pongo/tree/master/examples).

For a more complete example, you can view a simple Markdown pastebin project which includes Pongo [here](https://github.com/hkauso/sealable).

## Usage

You can add Pongo as a dependency using the command below:

```bash
cargo add pongo --no-default-features
```

The Pongo database should be created and passed into `pongo::dashboard::routes` to serve the dashboard and API together.

```rust
// https://github.com/hkauso/pongo/blob/master/examples/basic.rs#L19-L27
// create database
let database = Database::new(Database::env_options(), ServerOptions::truthy()).await;
database.init().await;

// create app
let app = Router::new()
  .nest("/@pongo", pongo::dashboard::routes(database.clone()))
  .nest_service("/static", get_service(ServeDir::new("./static")))
  .fallback(pongo::api::not_found);
```

Pongo should be nested under the `/@pongo` path for the dashboard to work properly.

You can configure the location where Pongo will send its static file requests to through the `PO_STATIC_DIR` environment variable. This **DOES NOT** set the location that the local directory is served from, that is up to you!

Pongo expects an available Redis server to be running so that the dashboard can allow Redis key management. Pongo best integrates into other Dorsal-based servers since it'll pull the same database configuration.
