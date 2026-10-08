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

# Only the app gets a bundle. Cargo runs test binaries through this runner
# too, and a bundle around one would register a second app with this
# identifier, which Launch Services may then hand the files the app opens.
case "$(basename -- "$binary")" in
  screenwide) ;;
  *) exec "$binary" "$@" ;;
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
# The app looks for the programs and the voice activity model it bundles
# beside its own executable, which here is the link inside the bundle. A
# release puts them in the bundle too, so the prepared copies beside the build
# are linked there.
for tool in ffmpeg screenwide-transcriber silero-vad.bin; do
  if [ -e "$(dirname -- "$binary")/$tool" ]; then
    ln -sfn "$(dirname -- "$binary")/$tool" "$app_directory/Contents/MacOS/$tool"
  fi
done

exec "$app_executable" "$@"
