{ pkgs, lib, config, inputs, ... }:

{
  cachix.enable = false;

  packages = [ 
    pkgs.git 
    pkgs.cargo-espflash
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "cargo" "rustc" "rust-src" "rustfmt" ];
    targets = [ "riscv32imac-unknown-none-elf" ];
  };

  scripts.banner.exec = ''
    cat banner.txt
  '';

  enterShell = ''
    banner
  '';

  # See full reference at https://devenv.sh/reference/options/
}