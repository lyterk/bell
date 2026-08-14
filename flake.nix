{
  description = "Bell - random audio reminder";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "bell";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.alsa-lib ];

          postInstall = ''
            mkdir -p $out/share/bell/resources
            cp -r resources/* $out/share/bell/resources/
          '';
        };
      }
    )
    // {
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        {
          options.services.bell = {
            enable = lib.mkEnableOption "bell audio reminder service";
            resourcesDir = lib.mkOption {
              type = lib.types.str;
              default = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/share/bell/resources";
              description = "Path to directory containing audio files";
            };
            user = lib.mkOption {
              type = lib.types.str;
              default = "bell";
              description = "User to run the service as";
            };
          };

          config = lib.mkIf config.services.bell.enable {

            systemd.user.services.bell = {
              description = "Bell audio reminder";
              wantedBy = [ "default.target" ];

              serviceConfig = {
                ExecStart = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/bell";
                WorkingDirectory = config.services.bell.resourcesDir;
                Restart = "always";
                RestartSec = "5s";
              };
            };
          };
        };
    };
}
