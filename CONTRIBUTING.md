# Contributing to Bob

Thanks for wanting to contribute! Bob is community-driven and we appreciate all contributions.

## Quick Links

- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Guidelines](#guidelines)
- [Submitting Changes](#submitting-changes)
- [Questions?](#questions)

## How to Contribute

**Code:**

- Fix bugs or add features
- Improve performance

**Non-Code:**

- Improve docs
- Report bugs
- Suggest features
- Help answer issues
- Write tutorials

## Development Setup

### Prerequisites

- Rust toolchain ([rustup](https://rustup.rs/))
- Git

### Optional: Building Neovim

Only needed if testing the build-from-source feature. Requires CMake, Git, and platform-specific build tools. See the [Neovim wiki](https://github.com/neovim/neovim/wiki/Building-Neovim).

### Optional (but recommended): Installing and Using `cargo-make`

If you're planning on implementing changes that would warrant the classic "code-compile-change" development cycle,
the project is setup to use two tools in tandem:

- [Cargo Make](https://github.com/sagiegurari/cargo-make) and,
- [Taplo TOML formatter](https://taplo.tamasfe.dev/)

Cargo Make can greatly speed up the development cycle and other common actions.
Taplo is used by the `Makefile.toml` when formatting the project to keep all those contributing using
the same formatting.

#### If you're on Linux

Use your package manager to search for `cargo-make` and install, if available.

- Example for Arch Linux from the `cargo-make` docs:

```bash
paru -S cargo-make taplo-cli
```

#### If you're on Windows

For `cargo-make`:

You can use either `cargo-binstall` or simply `cargo` itself.

```powershell
cargo binstall cargo-make
```

or via cargo directly

```powershell
cargo install --force cargo-make
```

For `taplo-cli`:

```powershell
scoop install taplo
```

Or refer to the Taplo documentation directly.

You can access and use cargo-make via either `cargo-make <reciple>` or `makers <reciple>`.

> Please refer to the top-level `Makefile.toml` for the various aliased commands available to you.
> Per cycle I've found using alias `makers a` to be the best. It runs formatting, then all tests, and builds
> the 2 output types (debug and release).
> There's also `makers r` and `makers rr` for `run` and `run --release` respectively.
> You can pass arguments to the built binary easily via `makers r -- --help`

### Get Started

```sh
# Fork and clone
git clone https://github.com/YOUR_USERNAME/bob.git
cd bob

# Create a branch
git checkout -b feature/your-feature

# Build using cargo-make
makers a # or cargo-make a
# or
# Build using cargo
cargo build

# Run via cargo-make
makers r -- --help # or cargo-make r -- --help
# or
# Run using cargo
cargo run -- --help
```

## Guidelines

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix issues
- Write idiomatic Rust
- Avoid `unwrap()` in production code
- Ensure cross-platform compatibility

### Testing

```
cargo test
```

If applicable add tests for new features and bug fixes. Test manually before submitting.

### Commit Format

```
<type>: <short summary>

feat: add rollback feature
fix: resolve Windows path issue
docs: update installation guide
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Submitting Changes

### The Rule: Issue First, PR Second

**All PRs must link to an existing, discussed issue.** Don't open PRs out of thin air.

**Workflow:**

1. Open an issue describing the change
2. Discuss with maintainers
3. Get approval
4. Code and open PR linking to the issue

**Exception:** Typos, and docs fixes can skip the issue.

### PR Checklist

- [ ] Tests pass
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Docs updated
- [ ] Links to issue with "Fixes #123" or "Closes #456"

### Bug Reports

Include:

- Bob version (`bob --version`)
- OS and architecture
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior
- Logs/errors
- Config file (if relevant)

### Feature Requests

Check existing issues first. Open a new issue explaining:

- What you need and why
- How it should work
- Alternative approaches you considered

## Questions?

- **Issues:** Bug reports and feature requests
- **Discussions:** Questions and general chat
- **Sponsor:** [Polar.sh](https://polar.sh/MordechaiHadad) or GitHub Sponsors

We'll respond within a few days. Patience appreciated—this is volunteer work!

## License

By contributing, you agree your code will be licensed under the MIT License.

---

Thanks for contributing! 🚀
