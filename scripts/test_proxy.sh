#!/usr/bin/env bash

set -euo pipefail

# BASEDIR=$(dirname "$0")

TEST_NAME="idc_about_my_name_anymore"
BUILT_TOPT="target/optimized"
BOB_LOCAL_SHARE="$HOME/.local/share/bob/nvim-bin"
QUET_STUB="/tmp/bob_symlink_test"

git_root=$(git rev-parse --show-toplevel)

# util stuff starts (there's a similar marker at the bottom)
## @util start

catch_errors() {
  printf "An error occurred in script at line: %s\n" "$1"

  if command -v gum >/dev/null && gum confirm "You wanna try that again or wot?"; then
    bash "$0"
  else
    printf "Exiting now. bi bi\n"
  fi
  return 1
}
trap 'catch_errors $LINENO' ERR
trap 'trap - ERR' EXIT

function print_startup {
  requested_help="$1"
  if [ "$requested_help" = true ]; then
    :
  else
    [ -f "$QUET_STUB" ] && return 0
  fi

  # skip of out stub exists
  cat <<EOF

=============================
It's time to t-t-t-test how symlinks work!
=============================

This script will create a (new) symlink under 
your '$BOB_LOCAL_SHARE' directory
pointing to a freshly built bob proxy nvim binary.

The symlink is called: $TEST_NAME

Assuming that above directory is already on your PATH,
you can run it like so:

  $TEST_NAME --version

Or you can just call it from anywhere like so:

  $TEST_NAME

If you're able to still call the binary
and get through to real nvim despite 
it being called $TEST_NAME,
then the symlink test is a success!

To cleanup the test setup/resources, 
run this script with any of: 

  '-c' / '--clean' / clean

like so:

  bash $0 clean

You can call this help again with:

  bash $0 --help
EOF

  touch "$QUET_STUB" >/dev/null 2>&1 || {
    printf "Could not create temp file, aborting.\n" >&2
    exit 1
  }

  return 0
}

function 05print_end {
  cat <<EOF

=============================

You can now use the name you changed it to to start 
nvim (instead of it calling bob).

Assuming it's working correctly, it will 
now resolve correctly regardless of the
symlink's name under the new directory

$BOB_LOCAL_SHARE/$TEST_NAME

Have fun 

* remember you can cleanup with: 

  bash $0 clean

=============================

EOF

  return 0
}

function print_vars {

  cat <<EOF

=============================

Variables used in this script:

- root of the git repo
* "git_root" : $git_root

- build topt (target/optimized)
* "build_topt" : $BUILT_TOPT

- test name for the symlink
* "test_name" : $TEST_NAME

- bob local share directory for nvim binaries
* "BOB_LOCAL_SHARE" : $BOB_LOCAL_SHARE

- There's also a file generated in tmp 
to not run the help every time (for dev cycles)
* "QUET_STUB" : $QUET_STUB
EOF

  return 0
}

# util stuff ends
## @util end

function 00cargo_stuff {
  command cargo build &&
    cargo build --profile release &&
    cargo build --profile optimized
  return $?
}

function 01gen_proxy_copy {
  if [ ! -f "$git_root/$BUILT_TOPT/bob" ]; then
    printf "Built bob proxy binary not found at: %s\n" "$git_root/$BUILT_TOPT/bob"
    return 1
  fi

  # if it's there, remove it and recreate
  if [ -f "$git_root/$BUILT_TOPT/nvim" ]; then
    printf "Removing existing nvim proxy binary at: %s\n" "$git_root/$BUILT_TOPT/nvim"
    command rm -f "$git_root/$BUILT_TOPT/nvim"
  fi

  command cp "$git_root/$BUILT_TOPT/bob" "$git_root/$BUILT_TOPT/nvim"
  return $?
}

function 02backup_prd_proxy_nvim {
  # check the file we're about to try and move exists
  # if it doesn't that's fine, we can skip the move
  if [ ! -f "$BOB_LOCAL_SHARE/nvim" ]; then
    printf "No existing production proxy nvim binary found, skipping backup step.\n"
    return 0
  fi

  cat <<EOF

=============================

- Backing up existing production proxy nvim binary at: 
* "PRD Binary" : $BOB_LOCAL_SHARE/nvim

* Moving from: "$BOB_LOCAL_SHARE/nvim"
* To: "$BOB_LOCAL_SHARE/nvim_PRD"

=============================

EOF

  command mv "$BOB_LOCAL_SHARE/nvim" "$BOB_LOCAL_SHARE/nvim_PRD"
  return $?
}

function 03copy_proxy_to_localshare {
  command cp -f "$git_root/$BUILT_TOPT/nvim" "$BOB_LOCAL_SHARE/nvim"
  return $?
}

function 04symlink_proxy_to_test_name {

  place_in="$BOB_LOCAL_SHARE/$TEST_NAME"

  cat <<EOF

=============================

Creating symlink for proxy nvim binary...

- Real proxy nvim binary at: 
* "Real binary" : $BOB_LOCAL_SHARE/nvim

- Symlink to be created at:
* "Symlink" : $place_in

=============================

EOF

  declare change_loc
  read -p \
    "Do you want to change the location where the symlink will be created? (y/N): " \
    change_loc &&
    [[ $change_loc == [yY] || $change_loc == [yY][eE][sS] ]] || change_loc="N"

  if [ -z "$change_loc" ]; then
    change_loc="N"
  fi

  if [[ "$change_loc" =~ ^[Yy]$ ]]; then
    read -p "Enter the full path where you want the symlink to be created: " new_loc
    place_in="$new_loc"
    printf "Symlink will (now) be created at: %s\n" "$place_in"
  fi

  # command ln -sf "$git_root/$BUILT_OPT/nvim" "$BOB_LOCAL_SHARE/$TEST_NAME"
  command ln -sf "$BOB_LOCAL_SHARE/nvim" "$BOB_LOCAL_SHARE/$TEST_NAME"
  return $?
}

function revert_test {
  cat <<EOF

=============================

Cleaning up test proxy nvim setup...

- Removing symlink at: 
* "$BOB_LOCAL_SHARE/$TEST_NAME"
EOF

  rm -f "$BOB_LOCAL_SHARE/$TEST_NAME"

  cat <<EOF

=============================

- Removing test proxy nvim binary at:
* "$BOB_LOCAL_SHARE/nvim"

- Now swapping it back with the prod binary backup.

* Moving from: "$BOB_LOCAL_SHARE/nvim_PRD"
* To: "$BOB_LOCAL_SHARE/nvim"

=============================

EOF

  # check if the backup exists
  if [ ! -f "$BOB_LOCAL_SHARE/nvim_PRD" ]; then
    printf "No backup production proxy nvim binary found at: %s\n" "$BOB_LOCAL_SHARE/nvim_PRD"
    printf "Cleanup incomplete, please check manually.\n"
    return 1
  fi

  mv "$BOB_LOCAL_SHARE/nvim_PRD" "$BOB_LOCAL_SHARE/nvim"
  printf "Cleanup complete.\n"
  return 0
}

function main {
  # Parse command-line arguments
  case "${1:-}" in
  -h | --help | help)
    printf "Printing help...\n"
    print_startup true
    return 0
    ;;
  -v | --vars | vars)
    printf "Printing variable values...\n"
    print_vars
    return 0
    ;;
  -c | --clean | clean)
    printf "Reverting test setup...\n"
    revert_test
    return 0
    ;;
  *)
    print_startup false
    ;;
  esac

  arguments=(
    "00cargo_stuff"
    "01gen_proxy_copy"
    "02backup_prd_proxy_nvim"
    "03copy_proxy_to_localshare"
    "04symlink_proxy_to_test_name"
    "05print_end"
  ) # && printf "Going to run: %s\n" "${arguments[*]}"

  while [ ${#arguments[@]} -gt 0 ]; do
    "${arguments[0]}"
    if [ $? -ne 0 ]; then
      printf "Error occurred in step: %s\n" "${arguments[0]}"
      exit 1
    fi
    arguments=("${arguments[@]:1}")
  done
}

main "$@"
