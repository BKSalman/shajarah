{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs@{ self, nixpkgs, rust-overlay, flake-parts, crane, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
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

              port = lib.mkOption {
                type = lib.types.port;
                default = 8080;
                description = "Port the server listens on.";
              };

              configFile = lib.mkOption {
                type = lib.types.path;
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
                  SHAJARAH_CONFIG_PATH = builtins.dirOf cfg.configFile;
                  SHAJARAH_CONFIG_FILE = builtins.baseNameOf cfg.configFile;
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

                  # Hardening.
                  DynamicUser = true;
                  ProtectSystem = "strict";
                  ProtectHome = true;
                  PrivateTmp = true;
                  NoNewPrivileges = true;
                };
              };

              # NOTE: database migrations are not embedded in the binary. Run them
              # out-of-band before/after deploy, e.g.:
              #   sqlx migrate run --source ${./web/migrations}
            };
          };
      };

      perSystem = { system, lib, ... }:
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
              lock = builtins.fromTOML (builtins.readFile ./Cargo.lock);
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
        in
        {
          packages.default = shajarah;
          packages.shajarah = shajarah;

          devShells.default = pkgs.mkShell.override {
            stdenv = pkgs.useWildLinker pkgs.stdenv;
          } rec {
            packages = [
              # Rust
              (pkgs.rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "wasm32-unknown-unknown" ];
              })
              pkgs.dioxus-cli
              pkgs.cargo-watch
              pkgs.sqlx-cli
              pkgs.vscode-langservers-extracted
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

            shellHook = ''
              cleanup() {
                echo "Stopping development containers..."
                docker stop shajarah-dev-db shajarah-dev-pgweb shajarah-dev-mailhog &>/dev/null &
              }

              # Register cleanup on exit
              trap cleanup EXIT

              run-services() {
                if ! docker ps --format '{{.Names}}' | grep -q '^shajarah-dev-db$'; then
                  docker run --rm \
                    --name shajarah-dev-db \
                    -p 5445:5432 \
                    -e POSTGRES_PASSWORD=shajarah-dev-db \
                    -d postgres &> /dev/null
                fi

                if ! docker ps --format '{{.Names}}' | grep -q '^shajarah-dev-pgweb$'; then
                  docker run --rm \
                    --name shajarah-dev-pgweb \
                    -p 8081:8081 \
                    -d sosedoff/pgweb &> /dev/null
                fi

                if ! docker ps --format '{{.Names}}' | grep -q '^shajarah-dev-mailhog$'; then
                  docker run --rm \
                    --name shajarah-dev-mailhog \
                    -p 1025:1025 -p 8025:8025 \
                    -d mailhog/mailhog:v1.0.1 &> /dev/null
                fi

                ${pkgs.sqlx-cli}/bin/sqlx migrate run || ${pkgs.sqlx-cli}/bin/sqlx migrate run --source web/migrations
              }

              export DATABASE_URL=postgres://postgres:shajarah-dev-db@localhost:5445/postgres

              export $(cat .env)
              run-services
            '';
          };
        };
    };
}
