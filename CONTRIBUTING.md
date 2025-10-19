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
- OpenSSL (optional, for `native-tls` feature)

### Optional: Building Neovim

Only needed if testing the build-from-source feature. Requires CMake, Git, and platform-specific build tools. See the [Neovim wiki](https://github.com/neovim/neovim/wiki/Building-Neovim).

### Get Started

```
# Fork and clone
git clone https://github.com/YOUR_USERNAME/bob.git
cd bob

# Create a branch
git checkout -b feature/your-feature

# Build
cargo build

# Run
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