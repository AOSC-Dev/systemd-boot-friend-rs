#!/bin/bash -ex

PREFIX="${PREFIX:-/usr/local}"

# install completions
install -dv "${PREFIX}/share/zsh/functions/Completion/Linux/"
install -Dvm644 completions/_ciel "${PREFIX}/share/zsh/functions/Completion/Linux/"
install -dv "${PREFIX}/share/fish/completions/"
install -Dvm644 completions/ciel.fish "${PREFIX}/share/fish/completions/"
install -dv "${PREFIX}/share/bash-completion/completions/"
install -Dvm644 completions/ciel.bash "${PREFIX}/share/bash-completion/completions/"