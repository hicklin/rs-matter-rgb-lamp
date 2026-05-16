{
  description = "Rust development environment";

  inputs = {
    # Hydra-pinned nixpkgs for maximum cache availability
    # See available pins at: https://hydra.nixos.org/jobset/nixpkgs/trunk/evals
    # Using a hydra pin significantly speeds up CI by ensuring prebuilt binaries are available
    nixpkgs.url = "github:nixos/nixpkgs/8a1b0127302ea51e05bf4ea5a291743fac442406";
    # nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11"; # stable pin

    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Configuration: Rust nightly version
        # Update this date to change the Rust nightly version used across the project
        # Available nightlies: https://rust-lang.github.io/rustup-components-history/
        rustNightlyDate = "2026-05-15";

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Extract nixpkgs commit info for display
        nixpkgsRev = nixpkgs.rev or "unknown";
        nixpkgsShortRev = builtins.substring 0 7 nixpkgsRev;

        # Try to extract date from flake metadata (flake.lock stores lastModified)
        nixpkgsDate =
          let
            lastModified = nixpkgs.lastModified or 0;
          in
            if lastModified != 0
            then builtins.readFile (pkgs.runCommand "format-date" {} ''
              date -d @${toString lastModified} +%Y-%m-%d > $out
            '')
            else "unknown";

        rustToolchain = pkgs.rust-bin.nightly.${rustNightlyDate}.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" "rust-analyzer" "rustc"];
          targets = [
            "riscv32imac-unknown-none-elf"
          ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain

            # Rust tools
            pkgs.cargo-expand
            pkgs.cargo-generate

            # Embedded tools
            pkgs.espflash
            pkgs.esptool
            pkgs.probe-rs-tools
            pkgs.picotool

            # Version control
            pkgs.git
          ];

          shellHook = ''
            echo ""
            echo "═══════════════════════════════════════════════════════════"
            cat banner.txt
            echo ""
            echo "───────────────────────────────────────────────────────────"
            echo "📌 nixpkgs: ${nixpkgsShortRev} (${builtins.replaceStrings ["\n"] [""] nixpkgsDate})"
            echo "🦀 Rust: nightly ${rustNightlyDate}"
            echo "📦 Targets: "
            echo "       riscv32imac-unknown-none-elf"
            echo "───────────────────────────────────────────────────────────"
            echo "💡 To update nixpkgs or rust, consult the project README.md"
            echo "═══════════════════════════════════════════════════════════"
            echo ""
          '';
        };
      }
    );
}
