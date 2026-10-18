{
  inputs = {
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs-lib";
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";
    nixpkgs-lib.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-substituters = [ "https://fenix.cachix.org" ];
    extra-trusted-public-keys = [
      "fenix.cachix.org-1:ecJhr+RdYEdcVgUkjruiYhjbBloIEGov7bos90cZi0Q="
    ];
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];

      imports = [ inputs.treefmt-nix.flakeModule ];

      perSystem =
        {
          inputs',
          lib,
          pkgs,
          self',
          ...
        }:
        let
          toolchain = inputs'.fenix.packages.stable.toolchain;

          crane = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;

          fmtt = pkgs.writeShellApplication {
            name = "fmtt";
            text = ''${lib.getExe self'.formatter} "$@"'';
          };
        in
        {
          devShells.default = pkgs.mkShell {
            packages = [
              fmtt
              pkgs.cargo-expand
              pkgs.cargo-flamegraph
              pkgs.cargo-semver-checks
              pkgs.cargo-msrv
              pkgs.nixd
              toolchain
            ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt = {
                enable = true;
                excludes = [ ".direnv" ];
              };
              rustfmt = {
                enable = true;
                package = inputs'.fenix.packages.latest.rustfmt;
              };
              taplo.enable = true;
              typos.enable = true;
            };
          };

          checks =
            let
              args = {
                src = crane.cleanCargoSource ./.;
                strictDeps = true;
              };
            in
            {
              test = crane.cargoNextest (
                args
                // {
                  cargoArtifacts = crane.buildDepsOnly args;
                }
              );

              lint = crane.cargoClippy (
                args
                // {
                  cargoArtifacts = crane.buildDepsOnly args;
                }
              );
            };
        };
    };
}
