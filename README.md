# Shajarah | شجرة
A web app for managing and displaying your family tree

# Building from source
When building from source, you need to build the backend, and the egui app seperately:

to build the egui app:
```bash
trunk build

# or watch
trunk watch
```

to build the backend:
```bash
cd server

cargo build

# or watch
cargo watch -x r
```

# Deployment

## Deploying with Docker
this is still WIP (contributions are welcome!)

## Deploying with Nix
the `flake.nix` at the root of the repository provides the backend and the egui app together as the default derivation
so to deploy, you can use the flake, and run it as a systemd service in your nix config

example:
`flake.nix`
```
...
inputs = {
  ...
  shajarah.url = "github:bksalman/shajarah";
};

outputs = { shajarah, ... }: {
  nixosConfigurations.nixos = nixpkgs.lib.nixosSystem {
    system = "aarch64-linux"; # or "x86_64-linux"
    specialArgs = {inherit shajarah;};
    modules = [ ./configuration.nix ];
  };
}
...
```

`configuration.nix`
```nix
systemd.services.shajarah = {
  description = "Shajarah";
  after = [
    "network.target"
    "postgresql.service"
  ];
  wantedBy = [ "multi-user.target" ];
  environment = {
    DATABASE_URL = "postgres://postgres@localhost:5432/shajarah";
    RUST_LOG = "debug";
    SHAJARAH_CONFIG_PATH = "${config.sops.secrets.shajarah-config.path}";
  };
  serviceConfig = {
      Requires = "postgresql.service";
      ExecStart = "${shajarah.packages."aarch64-linux".default}/bin/server --address 0.0.0.0:8080";
  };
};
```
this will run the server and serve the egui app on `http://example.com/`
