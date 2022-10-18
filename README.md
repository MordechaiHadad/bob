<div align="center">

<img src="resources/bob-nvim-logo-2-transparent-bg.png" width=315>

</div>

# Bob

> Struggle to keep your Neovim versions in check? Bob provides an easy way to install and switch versions on any system!

Bob is a cross-platform and easy-to-use Neovim version manager, allowing for easy switching between versions right from the command line.

## Prerequisites

Make sure you don't have Neovim already installed via other ways e.g. a package manager.

### Building from source

Make sure you have [rustup](https://www.rust-lang.org/tools/install) installed.

## :wrench: Installation

### Install from releases

1. Download `bob-{platform}-x86_64.zip`
2. Unzip it
3. And now run it with `bob`

#### For Debian and Linux OpenSSL v3.0.+ users

Due to some weird reasoning bob requires OpenSSL v1.1(even thought it was built using an Ubuntu machine) users of OpenSSL v3.0.+ will need to use the `bob-linux-rustls-x86_64.zip` package.

This issue needs a little more researching to possibly fix, but for now it will stay like this.

### Install from source

#### For Linux OpenSSL v1.1 and windows/macos users

1. `cargo install --git https://github.com/MordechaiHadad/bob.git --features bob/openssl`

#### For OpenSSL v3.0.+ users

1. `cargo install --git https://github.com/MordechaiHadad/bob.git --features bob/rustls`
2. Run bob with `bob`

## :question: Usage

- `bob use |nightly|stable|<version-string>|<commit-hash>|`

Switch to the specified version, will auto-invoke install command if the version is not installed already.

**Windows side note:** make sure to run the application as administrator to properly switch a version.

---

- `bob install |nightly|stable|<version-string>|<commit-hash>|`

Install the specified version, can also be used to update out-of-date nightly version.

---

- `bob uninstall |nightly|stable|<version-string>|`

Uninstall the specified version.

---

- `bob erase`

Erase any change bob ever made including Neovim installation, Neovim version downloads and registry changes.

---

- `bob list`

List all installed and used versions.

---

## Showcase

A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`

<img src="https://user-images.githubusercontent.com/33547558/164478344-2707eb41-5b26-452e-ba05-c18282a3503a.gif">

## Configuration

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
  "downloads_dir": "/home/user/.local/share/bob/", // The folder in which neovim versions will be installed too, bob will error if this option is specfied but the folder doesn't exist
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
