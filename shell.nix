with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [
    cargo
    rustc
    rustfmt
  ];
}
