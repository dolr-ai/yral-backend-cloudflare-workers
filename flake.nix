{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
            inherit system overlays;
      };
      in rec {
       devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            yarn
            nodejs_22
            nodePackages.typescript-language-server
            nodePackages.eslint
            protobuf_21
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            curl
            worker-build
            openssl
            pkg-config
          ];
          shellHook = ''
            ./setup-hooks.sh
            '';
        };
      });
}
