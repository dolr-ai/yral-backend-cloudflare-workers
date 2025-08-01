{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
      in rec {
       devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            yarn
            nodejs_22
            nodePackages.typescript-language-server
            nodePackages.eslint
            protobuf_21
          ];
          shellHook = ''
            ./setup-hooks.sh
            '';
        };
      });
}
