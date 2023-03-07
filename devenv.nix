# https://devenv.sh/getting-started
{pkgs, ...}: {
  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    openssh
    just
    cargo-tarpaulin
    cargo-audit
    cargo-auditable
    cargo-insta # https://github.com/mitsuhiko/insta
    codespell

    # LSPs:
    rust-analyzer
    rnix-lsp
    marksman
    taplo
    sqls
    nodePackages.bash-language-server
    nodePackages.yaml-language-server
  ];

  enterShell = ''
    # set -x  # for debugging
    echo ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    echo "Run 'just' often, to prevent CI failures."
    echo ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  '';

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/pre-commit-hooks/
  # https://devenv.sh/reference/options/#pre-commit
  pre-commit.hooks = {
    commitizen.enable = true; # https://www.conventionalcommits.org/en/v1.0.0/#summary

    rustfmt.enable = true;
    cargo-check.enable = true;

    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;

    shfmt.enable = true;
    shellcheck.enable = true;

    mdsh.enable = true; # execute example shell from Markdown files
    yamllint.enable = true;
    html-tidy.enable = true;
  };
  pre-commit.settings = {
    yamllint.relaxed = true;
  };
}
