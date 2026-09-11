#!/bin/sh
# Keep the optional engine and all generated state inside this repository.
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export IMPECCABLE_HOME="$project_root/.impeccable/runtime"
cd "$project_root"
exec sh "$project_root/.agents/skills/impeccable/scripts/impeccable" "$@"
