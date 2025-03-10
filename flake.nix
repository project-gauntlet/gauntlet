{
  description = "https://github.com/project-gauntlet/gauntlet";
  outputs = inputs: inputs.flake-parts.lib.mkFlake {inherit inputs;} {imports = [./nix];};
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    crane.url = "github:ipetkov/crane";
  };
}
