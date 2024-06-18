{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    rust-analyzer
    rustfmt
    tree-sitter-grammars.tree-sitter-rust
  ];

  shellHook = ''
    ${pkgs.onefetch}/bin/onefetch
  '';
}
