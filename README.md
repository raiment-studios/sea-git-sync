# 🌊 sea-git-sync

Utility for bidirectionally syncing a monorepo folder to a remote git repository (without submodules or subtrees).

This can be useful for working in a private monorepo and publishing an individual project to a public repo. The syncing is bidirectional which means that the public repo can accept merge requests and these will be pulled back into the monorepo. In the case of errors or merge conflicts, the syncing is all done with standard `git` commands so the user is able to resolve complex situations manually.

## Status

Currently functional but has seen limited testing, especially for non-trivial projects. Please use with caution in production. Contributions welcome to address limitations.

## Installation

**Requires**: [Rust](https://rustup.rs/) to be installed for installation.

```bash
cargo install --git https://github.com/raiment-studios/sea-git-sync
```

## Usage

```bash
cd monorepo/subdir_123/my-project
sea-git-sync --remote git@github:yourcompany/my-project.git
```

## Developement

### Contributing

Please feel free to file issues and open pull requests on GitHub!

### Roadmap

-   [ ] Additional user testing
-   [ ] Better user notification of how to handle failed automatic merges

### History

Created in July 2025 as a Rust application based on a Bash script that was used in the private Raiment Studios monorepo.

## FAQ

Nothing so far!

## License

See [LICENSE](LICENSE).
