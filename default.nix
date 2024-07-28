{ lib
, rustPlatform
}:
rustPlatform.buildRustPackage rec {
  pname = "hwfetch";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;
}

