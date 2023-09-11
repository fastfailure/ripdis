# https://devenv.sh/getting-started
{pkgs, ...}: {
  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/packages/
  packages = with pkgs; [
    cargo-audit
    cargo-auditable
    cargo-insta # https://github.com/mitsuhiko/insta
    cargo-outdated
    cargo-tarpaulin
    codespell
    git
    just
    podman
    sfz
  ];

  # https://devenv.sh/pre-commit-hooks/
  # https://devenv.sh/reference/options/#pre-commit
  pre-commit.hooks = {
    commitizen.enable = true; # https://www.conventionalcommits.org/en/v1.0.0/#summary
    # Rust
    rustfmt.enable = true;
    cargo-check.enable = true;
    # Nix
    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;
    # Shell
    shfmt.enable = true;
    shellcheck.enable = true;
    # YAML
    yamllint.enable = true;
  };
  pre-commit.settings = {
    yamllint.relaxed = true;
  };
  enterShell = ''
    # set -x  # for debugging
    echo ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    echo "Run 'just help' for hints."
    echo ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  '';
}
