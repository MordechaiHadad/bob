# Bob

> Struggle to keep your nvim versions in check? Bob provides an easy way to install and switch versions on any system!

Bob is a fast and easy to use neovim version manager, easily switch between versions with ease.

## :star2: Features
Cross-Platform, fast, lightweight and 0 dependencies.

## Prerequisites
Make sure you don't have neovim already installed via other ways e.g. package manager.
<details>
<summary>When building from source</summary>
Install [rustup](https://www.rust-lang.org/tools/install)
</details>

## :wrench: Installation
### Install from source
1. `cargo install --git https://github.com/MordechaiHadad/bob.git`
2. Run bob with `bob`

## :question: Usage
- `bob use |nightly|stable|<version-string>|`
- `bob install |nightly|stable|<version-string>|`
- `bob uninstall |nightly|stable|<version-string>|`
- `bob ls`

A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`

<details>
<summary>Windows side note</summary>

Make sure to run the application as administator to properly install a version.
</details>

## :heart: Credits And Inspiration
- [nvm](https://github.com/nvm-sh/nvm) A node version manager
- [nvenv](https://github.com/NTBBloodbath/nvenv) A neovim version manager written by NTBBloodbath
