{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
  };

  outputs = inputs@{ self, nixpkgs, rust-overlay, flake-parts, crane, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.process-compose-flake.flakeModule
      ];

      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      # System-independent outputs.
      flake = {
        nixosModules.default = { config, lib, pkgs, ... }:
          let
            cfg = config.services.shajarah;
          in
          {
            options.services.shajarah = {
              enable = lib.mkEnableOption "the shajarah family-tree server";

              package = lib.mkOption {
                type = lib.types.package;
                default = self.packages.${pkgs.system}.default;
                defaultText = lib.literalExpression "shajarah.packages.\${system}.default";
                description = "The bundled shajarah package (server binary + public assets).";
              };

              user = lib.mkOption {
                type = lib.types.str;
                default = "shajarah";
                description = "User account under which Shajarah runs.";
              };

              group = lib.mkOption {
                type = lib.types.str;
                default = "shajarah";
                description = "Group under which Shajarah runs.";
              };

              port = lib.mkOption {
                type = lib.types.port;
                default = 8080;
                description = "Port the server listens on.";
              };

              configFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = ''
                  Path to the runtime `config.toml` (family name, base_url, email config, …).
                  Secrets are better supplied via {option}`services.shajarah.environmentFile`.
                '';
              };

              environmentFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = ''
                  Path to an EnvironmentFile holding secrets, e.g. `DATABASE_URL`,
                  `SHAJARAH_COOKIE_SECRET`, `SHAJARAH_TOTP_KEY`, `SHAJARAH_EMAIL_PASSWORD`.
                  Kept out of the Nix store.
                '';
              };

              logLevel = lib.mkOption {
                type = lib.types.enum [ "error" "warn" "info" "debug" "trace" ];
                default = "info";
                description = "Server log level (RUST_LOG / SHAJARAH_LOG_LEVEL).";
              };

              extraEnvironment = lib.mkOption {
                type = lib.types.attrsOf lib.types.str;
                default = { };
                description = "Extra environment variables for the service (e.g. SHAJARAH_BASE_URL).";
              };
            };

            config = lib.mkIf cfg.enable {
              systemd.services.shajarah = {
                description = "shajarah family-tree server";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" "postgresql.service" ];

                environment = {
                  # The config loader reads `${SHAJARAH_CONFIG_PATH}/${SHAJARAH_CONFIG_FILE}`.
                  SHAJARAH_CONFIG_PATH = lib.mkIf (cfg.configFile != null) (dirOf cfg.configFile);
                  SHAJARAH_CONFIG_FILE = lib.mkIf (cfg.configFile != null) (baseNameOf cfg.configFile);
                  SHAJARAH_PORT = toString cfg.port;
                  SHAJARAH_LOG_LEVEL = cfg.logLevel;
                  RUST_LOG = cfg.logLevel;
                } // cfg.extraEnvironment;

                serviceConfig = {
                  # The server resolves its `public/` assets relative to the working directory.
                  WorkingDirectory = cfg.package;
                  ExecStart = "${cfg.package}/server";
                  EnvironmentFile = lib.mkIf (cfg.environmentFile != null) cfg.environmentFile;
                  Restart = "on-failure";

                  User = cfg.user;
                  Group = cfg.group;

                  # Hardening.
                  ProtectSystem = "strict";
                  ProtectHome = true;
                  PrivateTmp = true;
                  NoNewPrivileges = true;
                };
              };

              users.users = lib.mkIf (cfg.user == "shajarah") {
                shajarah = {
                  inherit (cfg) group;
                  isSystemUser = true;
                };
              };

              users.groups = lib.mkIf (cfg.group == "shajarah") {
                shajarah = { };
              };

              # NOTE: database migrations are not embedded in the binary. Run them
              # out-of-band before/after deploy, e.g.:
              #   sqlx migrate run --source ${./web/migrations}
            };
          };
      };

      perSystem = { self', system, lib, config, ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          rustToolchainFor = p: p.rust-bin.stable.latest.default.override {
            # wasm32-unknown-unknown is required for the Dioxus web client build.
            targets = [ "wasm32-unknown-unknown" ];
          };
          craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchainFor);

          # `dx bundle` requires a `wasm-bindgen` CLI whose version exactly matches the
          # `wasm-bindgen` crate in Cargo.lock. nixpkgs only ships 0.2.121 and the `dx` wrapper
          # appends 0.2.118, so build the matching version and put it first on PATH.
          #
          # The version is read from Cargo.lock (single source of truth). The two hashes below
          # are content hashes for that exact version and must be updated when it bumps — Nix
          # prints the correct value on mismatch. Both are platform-independent.
          wasmBindgenVersion =
            let
              lock = fromTOML (builtins.readFile ./Cargo.lock);
            in
            (lib.findFirst (p: p.name == "wasm-bindgen")
              (throw "wasm-bindgen not found in Cargo.lock")
              lock.package).version;

          wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = wasmBindgenVersion;
              hash = "sha256-vO4RSxi/sMWxmsEs3GuljdMfIRSu75A+Q+c5wgYToRU=";
            };
            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-Inup6vvJSG5ghNyeDPyZbfZo4d0LsMG2OJfStoaeDBs=";
            };
          };

          unfilteredRoot = ./.;
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              # Static assets referenced via `asset!`, `include_bytes!`, `include_image!`.
              (lib.fileset.fileFilter
                (file: lib.any file.hasExt [
                  "html" "scss" "css" "ttf" "woff2" "ico" "svg" "png" "jpg" "jpeg" "webp"
                ])
                unfilteredRoot
              )
              ./web/assets
              ./gui/assets
              ./gui/fonts
              ./web/.sqlx
              ./web/migrations
              ./web/Dioxus.toml
            ];
          };

          commonArgs = {
            inherit src;
            version = "0.1.0";
            strictDeps = true;

            # Compile SQLx queries offline against web/.sqlx.
            SQLX_OFFLINE = "true";
            # egui's web clipboard support requires this cfg (mirrors CI).
            RUSTFLAGS = "--cfg=web_sys_unstable_apis";

            buildInputs = [
            ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          };

          # Build *just* the cargo dependencies once, to warm the cache.
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "shajarah-deps";
          });

          # The Dioxus fullstack bundle: `dx bundle` compiles both the wasm client
          # (which statically links the egui `gui` crate) and the native server binary.
          shajarah = craneLib.mkCargoDerivation (commonArgs // {
            inherit cargoArtifacts;
            pname = "shajarah";

            nativeBuildInputs = [ wasm-bindgen-cli pkgs.dioxus-cli pkgs.binaryen ];

            # This is the final artifact; we don't reuse its `target/` as a cargo-artifacts
            # cache, so don't pack the (huge) target dir into the output.
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              export HOME=$(mktemp -d)
              dx bundle --package web --platform web --fullstack --release --out-dir target/dx-out
            '';

            installPhaseCommand = ''
              mkdir -p $out
              cp -r target/dx-out/* $out/
            '';
          });

          diesel-guard = pkgs.rustPlatform.buildRustPackage rec {
            pname = "diesel-guard";
            version = "0.12.0";

            src = pkgs.fetchCrate {
              inherit pname version;
              hash = "sha256-sU6qKWlR44m19MeOEO/8L8R8dUJCUq6NbA4D3uJ15bM=";
            };

            cargoHash = "sha256-ZKJZnvv0EcGCljpjAVor9vt/eeN9ferw67z/RPTCcVU=";

            nativeBuildInputs = [ pkgs.pkg-config pkgs.rustPlatform.bindgenHook ];
            buildInputs = [ pkgs.postgresql ];   # for libpq / libpg_query's build

            # libpg_query sometimes needs this; add only if the build complains:
            # nativeBuildInputs = [ pkgs.pkg-config pkgs.cmake ];

            doCheck = false;   # skip if its test suite wants a live database
          };
        in
        {
          packages.default = shajarah;
          packages.shajarah = shajarah;

          devShells.default = 
            let
              pgcfg = config.process-compose."services".services.postgres.shajarah;
            in
          pkgs.mkShell.override {
            stdenv = pkgs.useWildLinker pkgs.stdenv;
          } rec {
            inputsFrom = [
              config.process-compose."services".services.outputs.devShell
            ];

            packages = [
              self'.packages.services

              # Rust
              (pkgs.rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "wasm32-unknown-unknown" ];
              })
              pkgs.dioxus-cli
              pkgs.cargo-watch
              pkgs.sqlx-cli
              pkgs.vscode-langservers-extracted
              diesel-guard
            ];

            buildInputs = with pkgs; [
              # misc. libraries
              openssl
              pkg-config

              # GUI libs
              libxkbcommon
              libGL
              fontconfig

              # wayland libraries
              wayland

              # x11 libraries
              libXcursor
              libXrandr
              libXi
              libX11
            ];

            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
            DATABASE_URL = "postgres://${pgcfg.superuser}@${pgcfg.listen_addresses}:${toString pgcfg.port}/shajarah";
          };

          process-compose."services" = { config, ... }: {
            imports = [
              inputs.services-flake.processComposeModules.default
            ];

            services.postgres."shajarah" = {
              enable = true;
              superuser = "shajarah";

              initdbArgs = [
                "--locale=C.utf8"
                "--encoding=UTF8"
              ];

              initialDatabases = [
                {
                  name = "shajarah";
                  schemas = [
                    (pkgs.writeText "shajarah-init.sql" ''
                      CREATE EXTENSION IF NOT EXISTS pg_trgm;
                    '')
                  ];
                }
              ];
            };

            settings.processes.pgweb =
              let
                pgcfg = config.services.postgres.shajarah;
              in
              {
                environment.PGWEB_DATABASE_URL = "postgres://${pgcfg.superuser}@${pgcfg.listen_addresses}:${toString pgcfg.port}/shajarah";
                command = pkgs.pgweb;
                depends_on."shajarah".condition = "process_healthy";
              };

            settings.processes.mailhog = {
              command = "${lib.getExe pkgs.mailhog}";
            };
          };
        };
    };
}
