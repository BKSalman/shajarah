{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        inherit (pkgs) lib;

        rustToolchainFor = p: p.rust-bin.stable.latest.default.override {
          # Set the build targets supported by the toolchain,
          # wasm32-unknown-unknown is required for trunk.
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchainFor);

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter
              (file: lib.any file.hasExt [ "html" "scss" "ttf" ])
              unfilteredRoot
            )
            (lib.fileset.maybeMissing ./assets)
            ./server/migrations
            ./server/.sqlx
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        # Native packages

        nativeArgs = commonArgs // {
          cargoExtraArgs = "--package=server";
          pname = "server";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly nativeArgs;

        server = craneLib.buildPackage (nativeArgs // {
          inherit cargoArtifacts;
          SHAJARAH_DIST = shajarah;
        });

        # Wasm packages

        wasmArgs = commonArgs // {
          pname = "shajarah";
          cargoExtraArgs = "--package=shajarah";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };

        cargoArtifactsWasm = craneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });

        shajarah = craneLib.buildTrunkPackage (wasmArgs // {
          pname = "shajarah";
          cargoArtifacts = cargoArtifactsWasm;
          trunkIndexPath = "index.html";
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;
        });
      in with pkgs; {
        devShells.default = mkShell.override {
            stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
          } rec {
          packages = [
            # Rust
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = ["wasm32-unknown-unknown"];
            })
            trunk
            cargo-watch
            sqlx-cli
            vscode-langservers-extracted
          ];

          buildInputs = [
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
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
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
              docker run --rm \
                --name shajarah-dev-db \
                -p 5445:5432 \
                -e POSTGRES_PASSWORD=shajarah-dev-db \
                -d postgres &> /dev/null || true

              docker run --rm \
                --name shajarah-dev-pgweb \
                -p 8081:8081 \
                -d sosedoff/pgweb &> /dev/null || true

              docker run --rm \
                --name shajarah-dev-mailhog \
                -p 1025:1025 -p 8025:8025 \
                -d mailhog/mailhog:v1.0.1 &> /dev/null || true


               ${sqlx-cli}/bin/sqlx migrate run
            }

             export DATABASE_URL=postgres://postgres:shajarah-dev-db@localhost:5445/postgres

            export $(cat .env)
          '';
        };

        packages.default = server;
      });
}
