<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<div align="center">
	<img height="75px" alt="Screenwide logo" src="./src-tauri/icons/ScreenwideLogo.png" />
</div>

<h1 align="center">Screenwide</h1>

<div align="center">
	<img alt="Latest GitHub release" src="https://img.shields.io/github/v/release/overpolish/screenwide?display_name=tag&label=release&labelColor=5C5C5C&color=282320" />
	<img alt="Platforms: macOS and Windows" src="https://img.shields.io/badge/platforms-macOS_%7C_Windows-282320?logo=tauri&labelColor=5C5C5C&logoColor=white" />
	<img alt="License: GPL-3.0-or-later and Elastic-2.0" src="https://img.shields.io/badge/license-GPL--3.0_%2B_Elastic--2.0-282320?labelColor=5C5C5C" />
</div>

<br />

<div align="center">
	Everything screen.
</div>

<br />

<div align="center">
	<img width="768" alt="A recording playing in the editor, zooming in where the clicks and typing are" src="./.github/assets/demo.gif" />
</div>

<br />

# Features

- **Screen recording.** Record a display, a window or part of the screen, with your camera, microphone and the computer's sound or just one app's. A replay buffer keeps the last 30 seconds, to save when something worth keeping has just happened.
- **Moments.** Mark the points worth keeping as you record, of kinds you name, and hold the key to add a voice note.
- **Editing.** Cut a recording on a strip that shows what happened in it: what was said, clicks, typing and changes on screen. Move between scenes of screen and camera, zoom in by hand or let auto zoom follow the action, and set it on a background, your own wallpapers included.
- **Captions and transcripts.** Transcribe what was said, on your computer, show it as captions, and edit the recording by taking words out of its script.
- **Studio sound.** Clean up a voice, and lower whatever plays under it.
- **Screenshots.** Capture an area, a window or a scrolling page, combine captures, and mark them up.
- **Better cursors.** Make the cursor easier to follow, with control over its size and movement.
- **Keyboard shortcuts.** Show pressed shortcuts on the recording.
- **Text recognition.** Copy text from anywhere on screen. QR codes work too.

## Paid tools

Each is free for 14 days, then bought on its own or all together from Settings › Licences.

- **Ruler.** Measure sizes, spacing and alignment on screen, and copy the result.
- **Annotate.** Draw arrows, counters, highlights and a magnifier on the screen as you present or record.
- **Glide.** Move and resize windows with a fluid trackpad or mouse gesture. Snap them into halves, quarters, thirds or full screen, or glide down to minimize.
- **References.** Keep a screenshot on screen at its true size, where it was taken, to compare against. Drag a picture with Option (Alt on Windows) to keep it the same way.

<br />

# Roadmap

<div>
	<h3>Live Transcription</h3>
	Transcribe live without keeping an audio recording.
</div>

<div>
	<h3>Custom Glide Grids</h3>
	Create custom Glide grid layouts beyond 2×2 and 3×3.
</div>

<div>
	<h3>Shader Filters</h3>
	GLSL shader support for filters (CRT)/custom backgrounds.
</div>

<br />

# Contributing

Setting up, running and checking the app on macOS and Windows: [CONTRIBUTING.md](CONTRIBUTING.md).

<br />

# License

Screenwide is free software under the [GNU General Public License, version 3 or later](LICENSE). Its paid tools, and the licence check they sit behind, are under the [Elastic License 2.0](LICENSES/Elastic-2.0.txt): you can read and change their source, but not remove or get round their licence key check, or offer them to others as a service. Buying a tool grants the [Screenwide Tools Licence](TOOLS-LICENSE.md). [`scripts/check-spdx.mjs`](scripts/check-spdx.mjs) lists the paths each licence covers, and every file's SPDX header names its licence. FFmpeg ships beside the app as a separate GPL program ([FFMPEG.md](FFMPEG.md)).
