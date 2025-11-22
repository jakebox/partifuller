{ pkgs }:

pkgs.rustPlatform.buildRustPackage {
  pname = "partifuller";
  version = "0.1.0";

  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl pkgs.sqlite ];

  RUSTFLAGS = [ "--cfg" "sqlx_macros_offline" ];
}
