{
  description = "A start menu for Wayland-based window managers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        libPath = with pkgs; lib.makeLibraryPath [ wayland vulkan-loader ];

        commonArgs = with pkgs; {
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              (lib.fileset.maybeMissing ./assets)
            ];
          };
          strictDeps = true;

          nativeBuildInputs = [ pkg-config makeWrapper ];
          buildInputs = [ fontconfig libxkbcommon ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "waystart";
        });

        waystartClippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        waystart = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            postInstall = ''
              wrapProgram "$out/bin/waystart" --prefix LD_LIBRARY_PATH : "${libPath}"
            '';
          }
        );
      in
      {
        checks = { inherit waystart waystartClippy; };
        packages.default = waystart;
        apps.default = flake-utils.lib.mkApp {
          drv = waystart;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [ rust-analyzer pkg-config fontconfig libxkbcommon ];
          LD_LIBRARY_PATH = libPath;
        };
      }
    )
    // {
      homeModules.default = import ./hm-module { inherit self; };
    };
}
