<div align="center">

<img src="resources/bob-nvim-logo-2-transparent-bg.png" width=315>

</div>

# Bob

> Struggle to keep your Neovim versions in check? Bob provides an easy way to install and switch versions on any system!

Bob is a cross-platform and easy-to-use Neovim version manager, allowing for easy switching between versions right from the command line.

## 🌟 Showcase

<img src="./resources/tapes/demo.gif">

## 🔔 Notices

- **2022-10-29**: Moved bob's symbolic link and downloads folder on macos from `/Users/user/Library/Application Support` to `~/.local/share` please make sure to move all of your downloads to the new folder, run `bob use <your desired version>` and update your PATH
- **2023-02-13**: Bob has recently switched to using a proxy executable for running Neovim executables. To switch from the old method that Bob used, follow these steps:

    1. Remove the current Neovim path from your global $PATH environment variable.
    2. Delete the following directory:
        On Unix: `~/.local/share/neovim`
        On Windows: `C:\Users\<username>\AppData\Local\neovim`

    Secondly the name of the downloads directory property in the configuration file has changed. Please refer to the updated list of properties for the new name.

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

### Install from crates.io

1. `cargo install bob-nvim`
2. Run bob with `bob`

## ❓ Usage

A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`

---

- `bob use |nightly|stable|latest|<version-string>|<commit-hash>|`

`--no-install` flag will prevent bob from auto invoking install command when using `use`

Switch to the specified version, by default will auto-invoke install command if the version is not installed already

---

- `bob install |nightly|stable|latest|<version-string>|<commit-hash>|`

Install the specified version, can also be used to update out-of-date nightly version.

---

- `bob sync`

If Config::sync_version_file_path is set, the version in that file will be parsed and installed.

---

- `bob uninstall |nightly|stable|latest|<version-string>|<commit-hash>|`

Uninstall the specified version.

---

- `bob rollback`

Rollback to an existing nightly rollback

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

| Property                       | Description                                                                                                                                                    | Default Value                                                                                                 |
| -------------------------------| ---------------------------------------------------------------------------------------------------------------------------------------------------------------| --------------------------------------------------------------------------------------------------------------|
| **enable_nightly_info**        | Will show new commits associated with new nightly release if enabled                                                                                           | `true`                                                                                                        |
| **downloads_location**         | The folder in which neovim versions will be downloaded to, bob will error if this option is specified but the folder doesn't exist                             | unix: `/home/<username>/.local/share/bob`, windows: `C:\Users\<username>\AppData\Local\bob`                   |
| **installation_location**      | The path in which the proxied neovim installation will be located in                                                                                           | unix: `/home/<username>/.local/share/bob/nvim-bin`, windows: `C:\Users\<username>\AppData\Local\bob\nvim-bin` |
| **version_sync_file_location** | The path to a file that will hold the neovim version string, useful for config version tracking, bob will error if the specified file is not a valid file path | `Disabled by default`                                                                                         |
| **rollback_limit**             | The amount of rollbacks before bob starts to delete older ones, can be up to 255                                                                               | `3`                                                                                                           |


### Example 

```jsonc
// /home/user/.config/bob/config.json
{
  "enable_nightly_info": true, // Will show new commits associated with new nightly release if enabled
  "downloads_location": "$HOME/.local/share/bob", // The folder in which neovim versions will be installed too, bob will error if this option is specified but the folder doesn't exist
  "installation_location": "/home/user/.local/share/bob/nvim-bin", // The path in which the used neovim version will be located in
  "version_sync_file_location": "/home/user/.config/nvim/nvim.version", // The path to a file that will hold the neovim version string, useful for config version tracking, bob will error if the specified file is not a valid file path
  "rollback_limit": 3 // The amount of rollbacks before bob starts to delete older ones, can be up to 225
}
```

## 🛠️ Troubleshooting

`sudo: nvim: command not found`
This error can be caused when `secure_path` is enabled in `/etc/sudoers` like in distros such as Fedora Workstation 37, possible workarounds:

1. disable `secure_path`
2. run `sudo env "PATH=$PATH" nvim`
3. set `$SUDO_USER` to location of bob nvim binary: `SUDO_EDITOR='/home/user/.local/share/bob/nvim-bin/nvim`

These workarounds were devised by @nfejzic, thanks to him.


## :heart: Credits And Inspiration

- [nvm](https://github.com/nvm-sh/nvm) A node version manager
- [nvenv](https://github.com/NTBBloodbath/nvenv) A Neovim version manager written by NTBBloodbath
