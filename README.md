# 🌊 sea-git-publish

Utility for bidirectionally syncing a monorepo folder to a remote git repository (without submodules or subtrees).

This can be useful for working in a private monorepo and publishing an individual project to a public repo. The syncing is bidirectional which means that the public repo can accept merge requests and these will be pulled back into the monorepo.

In the case of errors or merge conflicts, the syncing is all done with standard `git` commands so the user is able to resolve complex situations manually.

## License

See [LICENSE](LICENSE).
