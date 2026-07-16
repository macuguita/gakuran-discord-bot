{
  description = "Gakuran discord bot flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          gakuran-bot = pkgs.callPackage ./nix/package.nix { };
        in
        {
          inherit gakuran-bot;
          default = gakuran-bot;
        }
      );

      nixosModules.default = import ./nix/module.nix self;
      nixosModules.gakuran-bot = import ./nix/module.nix self;

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              nixd
              nixfmt
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              sqlx-cli
            ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

            shellHook = ''
              if [[ -f .env ]]; then
                source .env
              else
                echo "Could not find the .env file"
              fi

              if [[ -n "$DATABASE_URL" ]]; then
                if [[ ! -f gakuran-bot.db ]]; then
                  echo "Setting up database for compile-time checks..."
                  sqlx database create
                  sqlx migrate run
                else
                  sqlx migrate run
                fi
              else
                echo "DATABASE_URL not set, skipping database setup"
              fi
            '';
          };
        }
      );
    };
}
