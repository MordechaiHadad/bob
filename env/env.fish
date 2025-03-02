# Credit: https://github.com/rust-lang/rustup/blob/master/src/cli/self_update/env.fish
if not contains "{nvim_bin}" $PATH
    set -x PATH "{nvim_bin}" $PATH
end