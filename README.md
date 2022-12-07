<div align="center">

<img src="resources/bob-nvim-logo-2-transparent-bg.png" width=315>

</div>

# Bob

> Struggle to keep your Neovim versions in check? Bob provides an easy way to install and switch versions on any system!

Bob is a cross-platform and easy-to-use Neovim version manager, allowing for easy switching between versions right from the command line.

## 🌟 Showcase

<img src="https://user-images.githubusercontent.com/33547558/206171339-d256de56-e100-4ad5-a212-d1fa45df487d.gif">

## 🔔 Notices

- **2022-10-29**: Moved bob's symbolic link and downloads folder on macos from `/Users/user/Library/Application Support` to `~/.local/share` please make sure to move all of your downloads to the new folder, run `bob use <your desired version>` and update your PATH

## 📦 Requirements

Make sure you don't have Neovim already installed via other ways e.g. a package manager.

### Build prerequisites

#### Building bob

Make sure [rustup](https://www.rust-lang.org/tools/install) is installed.

#### Building Neovim

For further information refer to the [Neovim wiki](https://github.com/neovim/neovim/wiki/Building-Neovim#build-prerequisites).

<details>
<summary>All platforms</summary>

- CMake
- Git

</details>

<details>
<summary>Windows</summary>

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ extension pack

</details>

<details>
<summary>Unix</summary>

- Clang or GCC

**MacOS note**: [follow these instructions](https://github.com/neovim/neovim/wiki/Building-Neovim#macos--homebrew)

</details>

## 🔧 Installation

### Install from releases

1. Download `bob-{platform}-x86_64.zip`
2. Unzip it
3. Run it with `bob`

### Install from AUR

1. Install the [`bob`](https://aur.archlinux.org/packages/bob) or [`bob-bin`](https://aur.archlinux.org/packages/bob-bin) package with an AUR helper e.g. [paru](https://github.com/Morganamilo/paru): `paru -S bob`
2. Run it with `bob`

### Install from source

1. `cargo install --git https://github.com/MordechaiHadad/bob.git`
2. Run bob with `bob`

## Install from crates.io

1. `cargo install bob-nvim`
2. Run bob with `bob`

## ❓ Usage

A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`

---

- `bob use |nightly|stable|<version-string>|<commit-hash>|`

Switch to the specified version, will auto-invoke install command if the version is not installed already.

**Windows side note:** make sure to run the application as administrator to properly switch a version.

---

- `bob install |nightly|stable|<version-string>|<commit-hash>|`

Install the specified version, can also be used to update out-of-date nightly version.

---

- `bob uninstall |nightly|stable|<version-string>|<commit-hash>|`

Uninstall the specified version.

---

- `bob erase`

Erase any change bob ever made including Neovim installation, Neovim version downloads and registry changes.

---

- `bob list`

List all installed and used versions.

---

## ⚙ Configuration

This section is a bit more advanced and thus the user will have to do the work himself since bob doesn't do that.

Bob's configuration file will have to be in `config_dir/bob/config.json`, to be more specific:

<details>
<summary>On Linux</summary>

`/home/user/.config/bob/config.json`

</details>
<details>
<summary>On Windows</summary>

`C:\Users\user\AppData\Roaming\bob\config.json`

</details>
<details>
<summary>On MacOS</summary>

`/Users/user/Library/Application Support/bob/config.json`

</details>

### Syntax

```jsonc
// /home/user/.config/bob/config.json
{
  "enable_nightly_info": true, // Will show new commits associated with new nightly release if enabled
  "downloads_dir": "/home/user/.local/share/bob/", // The folder in which neovim versions will be installed too, bob will error if this option is specified but the folder doesn't exist
  "installation_location": "/home/user/.local/share/neovim" // The path in which the used neovim version will be located in
}
```

## :heart: Credits And Inspiration

- [nvm](https://github.com/nvm-sh/nvm) A node version manager
- [nvenv](https://github.com/NTBBloodbath/nvenv) A Neovim version manager written by NTBBloodbath

### Contributors

<table>
    <tr>
        <td align="center"><a href="https://github.com/max397574"><img src="https://avatars.githubusercontent.com/u/81827001?v=4" width="100px;" alt ""/><br/><sub><b>max397</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
        <td align="center"><a href="https://github.com/shift-d"><img src="https://avatars.githubusercontent.com/u/53366878?v=4" width="100px;" alt ""/><br/><sub><b>shift-d</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
        <td align="center"><a href="https://github.com/tamton-aquib"><img src="https://avatars.githubusercontent.com/u/77913442?v=4" width="100px;" alt ""/><br/><sub><b>Aquib</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
        <td align="center"><a href="https://github.com/vsedov"><img src="https://avatars.githubusercontent.com/u/28804392?v=4" width="100px;" alt ""/><br/><sub><b>Viv Sedov</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
        <td align="center"><a href="https://github.com/TarunDaCoder"><img src="https://avatars.githubusercontent.com/u/77536695?v=4" width="100px;" alt ""/><br/><sub><b>Tarun</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
        <td align="center"><a href="https://github.com/TheChoudo"><img src="https://avatars.githubusercontent.com/u/68950943?v=4" width="100px;" alt ""/><br/><sub><b>Dev Choudhuri</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a><a href="https://github.com/MordechaiHadad/bob/" title="README">📖</a><a href="https://github.com/MordechaiHadad/bob/" title="Logo">🎨</a></td>
        <td align="center"><a href="https://github.com/bryant-the-coder"><img src="https://avatars.githubusercontent.com/u/92417638?v=4" width="100px;" alt ""/><br/><sub><b>Bryant</b></sub></a><br /><a href="https://github.com/MordechaiHadad/bob/" title="Testing">👷</a></td>
    </tr>
</table>
