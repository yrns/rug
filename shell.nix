{ pkgs ? import <nixpkgs> {} }:

with pkgs; mkShell {
  # test runner requires python
  buildInputs = [ python ];
}
