{
  description = "tiny websocket server that broadcasts your currently playing last.fm song";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    eachSystem = f:
      nixpkgs.lib.genAttrs systems (system:
        f {
          inherit system;
          pkgs = import nixpkgs {inherit system;};
        });

    cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in {
    packages = eachSystem ({pkgs, ...}: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = cargoToml.package.name;
        inherit (cargoToml.package) version;
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        meta = {
          inherit (cargoToml.package) description;
          homepage = "https://github.com/tetra-fox/nowplaying";
          license = pkgs.lib.licenses.agpl3Plus;
          mainProgram = "nowplaying";
          platforms = pkgs.lib.platforms.unix;
        };
      };
    });

    apps = eachSystem ({system, ...}: {
      default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/nowplaying";
      };
    });

    devShells = eachSystem ({pkgs, ...}: {
      default = pkgs.mkShell {
        packages = with pkgs; [rustc cargo rustfmt clippy rust-analyzer];
      };
    });

    nixosModules.default = {
      lib,
      pkgs,
      ...
    }: {
      imports = [./module.nix];
      services.nowplaying.package = lib.mkDefault self.packages.${pkgs.stdenv.hostPlatform.system}.default;
    };
    nixosModules.nowplaying = self.nixosModules.default;

    formatter = eachSystem ({pkgs, ...}: pkgs.alejandra);
  };
}
