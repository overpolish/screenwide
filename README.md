<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<div align="center">
	<img height="75" alt="" src="./src-tauri/icons/ScreenwideLogo.png" />
	<h1>Screenwide</h1>
	<p>Everything screen.</p>
	<p><a href="https://getscreenwide.com">getscreenwide.com</a></p>
	<img alt="Latest GitHub release" src="https://img.shields.io/github/v/release/overpolish/screenwide?display_name=tag&label=release&labelColor=5C5C5C&color=282320" />
	<img alt="Platforms: macOS 14.2 or later and Windows" src="https://img.shields.io/badge/platforms-macOS_14.2%2B_%7C_Windows-282320?labelColor=5C5C5C" />
	<img alt="License: GPL-3.0-or-later and Elastic-2.0" src="https://img.shields.io/badge/license-GPL--3.0_%2B_Elastic--2.0-282320?labelColor=5C5C5C" />
</div>

<br />

<img alt="A recording playing in the editor, zooming in where the clicks and typing are" src="./.github/assets/demo.gif" />

## Recording

Record a display, a window or part of the screen, with your camera, microphone and the computer's sound. A replay buffer keeps the last 30 seconds, for when something worth keeping has just happened.

## Beats and lulls

The timeline splits a recording into beats, where something happens, and lulls, where nothing does. One handle squeezes every lull at once while the beats play at full speed. A lull can also be cut, kept as recorded or trimmed to a breath.

<img alt="The lulls on the timeline squeezed with one handle, the recording getting shorter as they speed up" src="./.github/assets/squeeze.gif" />

## Annotations

Magnify a detail, put one part in focus, spotlight or redact it, and add arrows, counters, highlights and text, on recordings and screenshots alike.

<table>
	<tr>
		<td><img alt="A magnifier on a chart's peak and a focus on one card, added to a recording" src="./.github/assets/annotations-recording.gif" /><br />On a recording</td>
		<td><img alt="Counters aimed at a headline and its subtitle, and a magnifier on a button, added to a screenshot" src="./.github/assets/annotations-screenshot.gif" /><br />On a screenshot</td>
	</tr>
</table>

## Free and paid

Recording, editing, screenshots and text recognition are free. Ruler, Annotate, Glide and References are paid tools: each is free for 14 days, then a one-off purchase from Settings › Licences, on its own or all together.

<table>
	<tr>
		<td><img alt="The Ruler checking a dialog's padding, field height and corner radius" src="./.github/assets/ruler.gif" /><br />Ruler</td>
		<td><img alt="An arrow, counters aimed at the figures, a highlight and a focus drawn over a dashboard" src="./.github/assets/annotate.gif" /><br />Annotate</td>
	</tr>
	<tr>
		<td><img alt="Two windows snapped into the halves of the screen" src="./.github/assets/glide.gif" /><br />Glide</td>
		<td><img alt="A reference of a design faded over the page as it was built" src="./.github/assets/references.gif" /><br />References</td>
	</tr>
</table>

## Download

Download Screenwide from [getscreenwide.com](https://getscreenwide.com), or get any version from [Releases](https://github.com/overpolish/screenwide/releases).

## Building

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

Screenwide is under the [GPL, version 3 or later](LICENSE). The paid tools and their licence check are under the [Elastic License 2.0](LICENSES/Elastic-2.0.txt): you can read and change their source, but not remove the licence check or offer the tools as a service. Buying a tool gives you the [Screenwide Tools Licence](TOOLS-LICENSE.md). Each file's SPDX header says which licence covers it. FFmpeg ships beside the app as a separate GPL program ([FFMPEG.md](FFMPEG.md)).
