## Set up a development environment

Install [devenv](https://devenv.sh/getting-started/)
and [direnv](https://direnv.net/#basic-installation).

Run `just help`.

## Unit testing

See [insta](https://docs.rs/insta/latest/insta/).

## Git workflow

Please follow
[semantic versioning](https://semver.org/) directives and
[GitLab Flow](https://docs.gitlab.com/ee/topics/gitlab_flow.html) workflow.

Please make an appropriate use of merge and rebase, as explained in
[this article](https://medium.com/@porteneuve/getting-solid-at-git-rebase-vs-merge-4fa1a48c53aa).

Use rebase when pulling to avoid creating useless merge commits.
The best way is to set in your git config file:

```
[pull]
  rebase = interactive
```

Use `--no-ff` when merging a development branch to create a merge commit even
if master has not advanced.
