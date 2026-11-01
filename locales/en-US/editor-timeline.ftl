# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The recording timeline: playback, tracks, moments and editing tools.

editor-timeline-label = Recording timeline
# $rate is a playback speed, such as "1.5".
editor-timeline-rate = { $rate }×
editor-timeline-play-preview = Play preview
editor-timeline-pause-preview = Pause preview
# The transport's toggle that turns a drag across the strip into a stretch to cut; holding Option (Alt on Windows) does the same for one drag.
editor-timeline-cut = Cut
editor-timeline-cut-name = Cut a stretch of the recording
# The transport's toggle that brings out the Tracks island, where each recorded track is held in the export and the preview or left out.
editor-timeline-tracks = Tracks
editor-timeline-tracks-name = Choose the tracks to play and export
editor-timeline-audio-visualizer = Audio visualizer
editor-timeline-system-audio = System audio
editor-timeline-microphone = Microphone
# An audio track the recording did not name. $number counts from 1.
editor-timeline-audio-track = Audio { $number }

## Tools and zoom

# A part of the recording that was cut out, put back by pressing its marker
# over the ruler. $duration is its length, such as 2.4 sec.
editor-timeline-restore-cut = Restore { $duration } cut

## The strip: the recording drawn to scale as beats and the lulls between them, with its scenes, annotations and captions.

# A lull, a stretch with nothing worth watching at full speed. $duration is its recorded length, such as 3.2 sec.
editor-timeline-lull = Lull, { $duration }
# A moment flagged while recording. $moment is its kind's name, which the user chooses, such as Notable.
editor-timeline-moment = { $moment } moment
# $layout names the scene's layout, such as Side by side.
editor-timeline-scene = Scene, { $layout }
# Shown where dragging along the scene line would draw a scene.
editor-timeline-scene-new = New scene
# $tool names the annotation's tool, such as Arrow.
editor-timeline-annotation = { $tool } annotation
# A bar for an annotation pinned to the content under it, which follows that content as the recording moves. $tool names the annotation's tool, such as Arrow.
editor-timeline-annotation-pinned = { $tool } annotation, pinned to content
# The window on the overview round the part of the recording the strip shows.
editor-timeline-view = Part in view
# The handle at the end of the overview, past the time already squeezed, dragged in to play every lull faster.
editor-timeline-squeeze = Squeeze the lulls
# Drawn past the strip's end, in the band that accounts for what was taken out of the recording, next to the end of what plays: the time playing faster saves, by the squeeze or a lull retimed. $duration is that time, such as 4.2 sec.
editor-timeline-squeezed = { $duration } squeezed
# Drawn past the strip's end, in the band that accounts for what was taken out of the recording, after the squeezed time: the time the cuts left out. $duration is that time, such as 12 sec.
editor-timeline-deleted = { $duration } deleted
# A mark where the recording no longer plays in the order it was recorded, after a stretch was moved; pressed, it puts the stretch back.
editor-timeline-put-back = Put back where it was recorded
# A shortcut pressed while recording, drawn over the picture where it was pressed. $keys are its keys, joined with +, such as Command+S.
editor-timeline-shortcut = Shortcut { $keys }
# A mark where the recording is split, so something moved can land there; pressed, it takes the split away.
editor-timeline-remove-split = Remove the split
