#!/bin/sh
# Credit: https://github.com/rust-lang/rustup/blob/master/src/cli/self_update/env.sh
case ":${PATH}:" in
    *:"{nvim_bin}":*)
        ;;
    *)
        export PATH="{nvim_bin}:$PATH"
        ;;
esac