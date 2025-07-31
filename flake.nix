{
  description = "eframe devShell";

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
        devShells.default = mkShell rec {
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
            docker start shajarah-dev || \
              docker run \
              --name shajarah-dev \
              -p 5445:5432 \
              -e POSTGRES_PASSWORD=shajarah-dev \
              -d postgres

            grep DATABASE_URL .env || echo "DATABASE_URL=postgres://postgres:shajarah-dev@localhost:5445/postgres" >> .env

            export $(cat .env)
          '';
        };

        packages.default = server;
      });
}
