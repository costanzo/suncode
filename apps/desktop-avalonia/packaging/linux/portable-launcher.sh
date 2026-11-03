#!/bin/sh
set -eu
app_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
exec "$app_directory/SunCode" "$@"
