# https://devenv.sh/getting-started
{pkgs, ...}: {
  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    openssh
    just
    cargo-insta # https://github.com/mitsuhiko/insta
  ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/pre-commit-hooks/
  # https://devenv.sh/reference/options/#pre-commit
  pre-commit.hooks = {
    editorconfig-checker.enable = true;
    commitizen.enable = true; # https://www.conventionalcommits.org/en/v1.0.0/#summary
    # other types: docs, refactor, chore, test, revert

    rustfmt.enable = true;
    cargo-check.enable = true;

    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;

    shfmt.enable = true;
    shellcheck.enable = true;

    # markdownlint.enable = true;
    mdsh.enable = true; # execute example shell from Markdown files
    yamllint.enable = true;
    html-tidy.enable = true;
  };
  pre-commit.settings = {
    yamllint.relaxed = true;
  };
}

