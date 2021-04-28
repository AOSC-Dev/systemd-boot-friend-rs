#!/bin/bash -ex

PREFIX="${PREFIX:-/usr/local}"

# install completions
install -dv "${PREFIX}/share/zsh/functions/Completion/Linux/"
install -Dvm644 completions/_systemd-boot-friend "${PREFIX}/share/zsh/functions/Completion/Linux/"
install -dv "${PREFIX}/share/fish/completions/"
install -Dvm644 completions/systemd-boot-friend.fish "${PREFIX}/share/fish/completions/"
install -dv "${PREFIX}/share/bash-completion/completions/"
install -Dvm644 completions/systemd-boot-friend.bash "${PREFIX}/share/bash-completion/completions/"