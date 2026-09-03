#!/bin/sh
# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

set -eu

binary=$1
shift

case "$binary" in
  /*) ;;
  *) binary="$(pwd)/$binary" ;;
esac

script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace_directory=$(dirname -- "$script_directory")
app_directory="$(dirname -- "$binary")/Screenwide.app"
app_executable="$app_directory/Contents/MacOS/screenwide"
app_resources="$app_directory/Contents/Resources"

mkdir -p "$app_directory/Contents/MacOS" "$app_resources"
cp "$script_directory/macos-dev-info.plist" "$app_directory/Contents/Info.plist"
cp "$workspace_directory/src-tauri/icons/icon.icns" "$app_resources/icon.icns"

# `tauri dev` watches src-tauri, so compiling Assets.car here would modify a
# watched file and restart the app forever. Copy the checked-in catalog; icon
# changes are generated explicitly or by the release bundler.
if cp "$workspace_directory/src-tauri/icons/Assets.car" "$app_resources/Assets.car"; then
  /usr/libexec/PlistBuddy \
    -c "Add :CFBundleIconName string Screenwide" \
    "$app_directory/Contents/Info.plist"
  touch "$app_directory" "$app_directory/Contents/Info.plist"
  /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
    -f "$app_directory"
else
  rm -f "$app_resources/Assets.car"
  echo "Could not prepare the macOS 26 app icon; using icon.icns" >&2
fi
ln -sfn "$binary" "$app_executable"

exec "$app_executable" "$@"
