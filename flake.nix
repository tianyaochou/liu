{
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.devenv.url = "github:cachix/devenv";

  outputs = inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; }
    {
      imports = [ inputs.devenv.flakeModule ];
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, ... }:{
        devenv.shells.default = let dbname = "liu-feed"; in {
          env.DATABASE_URL = "postgresql:${dbname}";
          packages = with pkgs; [ openssl ];
          languages.rust = {
            enable = true;
          };
          services.postgres = {
            enable = true;
            initialDatabases = [ { name = dbname; } ];
          };
        };
      };
    };
}
