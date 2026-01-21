{
  description = "A beautiful TUI for monitoring battery and energy usage";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Read version from root Cargo.toml workspace definition
        cargoToml = pkgs.lib.importTOML ./Cargo.toml;
        version = cargoToml.workspace.package.version;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "jolt";
          inherit version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Specific to workspace structure, target the CLI package
          cargoBuildFlags = [ "-p" "jolt-tui" ];

          # Feature handling: disable macos default on Linux, enable linux feature
          buildNoDefaultFeatures = pkgs.stdenv.isLinux;
          buildFeatures = pkgs.lib.optionals pkgs.stdenv.isLinux [ "linux" ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            udev
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            darwin.apple_sdk.frameworks.IOKit
            darwin.apple_sdk.frameworks.CoreFoundation
          ];

          meta = with pkgs.lib; {
            description = "A beautiful TUI for monitoring battery and energy usage";
            homepage = "https://getjolt.sh";
            license = licenses.mit;
            mainProgram = "jolt";
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];
        };
      }
    );
}
