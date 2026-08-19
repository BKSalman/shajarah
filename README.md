# Shajarah | شجرة
A web app for managing and displaying your family tree

Shajarah is a Rust fullstack app built with [Dioxus](https://dioxuslabs.com/).
The interactive tree canvas is rendered with an embedded [egui](https://github.com/emilk/egui)
app (the `gui` crate), the web UI and backend live in the `web` crate, and types
shared between them live in the `shared` crate. The backend is powered by axum,
sqlx, and PostgreSQL.

# Building from source
Building from source requires the [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/) (`dx`),
a Rust toolchain with the `wasm32-unknown-unknown` target, and a running PostgreSQL database.

Copy the example config and adjust it for your setup:
```bash
cp web/config.toml.example web/config.toml
```

Point the server at your database (the server embeds its migrations and applies
them on startup, so the database just needs to exist):
```bash
export DATABASE_URL="postgres://postgres@localhost:5432/shajarah"
```

To build and serve the app (client wasm + server) for development:
```bash
dx serve --package web
```

To produce a release bundle (the wasm client statically links the egui `gui` crate,
plus the native server binary and public assets):
```bash
dx bundle --package web --release
```

If you use Nix, the `devShell` in `flake.nix` provides the full toolchain (`dx`,
`sqlx-cli`, `wasm-bindgen-cli`, a nightly Rust toolchain, and a local PostgreSQL
via process-compose):
```bash
nix develop

# to run some dev servers (postgres, pgweb, mailhog)
services
```

# Deployment

## Deploying with Docker
this is still WIP

## Deploying with Nix
the `flake.nix` at the root of the repository provides the bundled package (server
binary + public assets) as the default derivation, and a NixOS module
(`nixosModules.default`) to run it as a systemd service.

example:

`flake.nix`
```nix
...
inputs = {
  ...
  shajarah.url = "github:bksalman/shajarah";
};

outputs = { nixpkgs, shajarah, ... }: {
  nixosConfigurations.nixos = nixpkgs.lib.nixosSystem {
    system = "aarch64-linux"; # or "x86_64-linux"
    specialArgs = {inherit shajarah;};
    modules = [
      shajarah.nixosModules.default
      ./configuration.nix
    ];
  };
}
...
```

`configuration.nix`
```nix
{ config, ... }: {
...
  services.shajarah = {
    enable = true;
    port = 8080;
    # Runtime config (family name, base_url, email config, …).
    configFile = "/etc/shajarah/config.toml";
    # Secrets kept out of the Nix store: DATABASE_URL, SHAJARAH_COOKIE_SECRET,
    # SHAJARAH_TOTP_KEY, SHAJARAH_EMAIL_PASSWORD, …
    environmentFile = config.sops.secrets.shajarah-env.path;
    logLevel = "info";
  };

  services.postgresql = {
    enable = true;
    settings = {
      port = 5432;
    };
    ensureDatabases = [ "shajarah" ];
    ensureUsers = [
      {
        name = "shajarah";
        ensureDBOwnership = true;
      }
    ];

    initialScript = pkgs.writeText "init-sql-script" ''
      CREATE EXTENSION IF NOT EXISTS pg_trgm;
    '';
  };
...
}
```
this will run the server and serve the app on `http://localhost:8080`.

The server embeds its migrations and runs them against `DATABASE_URL` on startup,
so no separate migration step is needed at deploy time.
