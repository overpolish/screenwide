# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The recording timeline: playback, tracks, moments and editing tools.

editor-timeline-label = Recording timeline
editor-timeline-position = Recording position
editor-timeline-select-range = Select timeline range
editor-timeline-resize = Resize timeline
editor-timeline-preview-speed = Preview speed
editor-timeline-speed = Speed
# $rate is a playback speed, such as "1.5".
editor-timeline-rate = { $rate }×
editor-timeline-range-speed = Range playback speed
editor-timeline-segment-speed = Segment playback speed
editor-timeline-play-preview = Play preview
editor-timeline-pause-preview = Pause preview
editor-timeline-copy-frame = Copy Frame
editor-timeline-copy-frame-label = Copy current frame
editor-timeline-preparing-audio = Preparing audio tracks
editor-timeline-preparing-audio-label = Preparing audio preview
editor-timeline-audio-visualizer = Audio visualizer
editor-timeline-segment = Timeline segment { $number }
editor-timeline-system-audio = System audio
editor-timeline-microphone = Microphone
# An audio track the recording did not name. $number counts from 1.
editor-timeline-audio-track = Audio { $number }

## Tools and zoom

editor-timeline-blade = Blade
editor-timeline-blade-name = Blade tool
editor-timeline-range = Range
editor-timeline-range-name = Range tool
editor-timeline-snap = Snap
editor-timeline-snap-name = Snap to edges
editor-timeline-zoom-out = Zoom out
editor-timeline-zoom-out-label = Zoom timeline out
editor-timeline-zoom-in = Zoom in
editor-timeline-zoom-in-label = Zoom timeline in
editor-timeline-fit = Fit timeline

## Tracks. $track names a track.

editor-timeline-shortcuts = Shortcuts
editor-timeline-annotations = Annotations
editor-timeline-scenes = Scenes
editor-timeline-camera-separate = Saved as its own file
# $note says how the track is exported.
editor-timeline-track-note = { $track }, { $note }
editor-timeline-track-include = Include { $track }
editor-timeline-track-exclude = Exclude { $track }
editor-timeline-track-required = { $track } must remain included
editor-timeline-track-required-tooltip = The export has to keep at least one track

## Moments: points flagged while recording. $moment is a kind's name, which
## the user chooses; $time where it sits in the recording.

editor-timeline-pin = { $moment }, { $time }
editor-timeline-pin-note = { $moment }, { $time }, with a voice note
# $text is the start of what the voice note says.
editor-timeline-pin-transcript = { $moment }, { $time }: { $text }
editor-timeline-go-to = Go to { $moment } moment at { $time }
editor-timeline-go-to-note = Go to { $moment } moment at { $time } and open its voice note
editor-timeline-note = { $moment } voice note
editor-timeline-play-note = Play voice note
editor-timeline-pause-note = Pause voice note
editor-timeline-no-speech = No speech in this note
editor-timeline-transcribe-failed = Could not transcribe this note
editor-timeline-transcribing = Transcribing
editor-timeline-transcribe-waiting = Waiting to transcribe
editor-timeline-no-model = No model downloaded
editor-timeline-download = Download
editor-timeline-downloading-model = Downloading model
editor-timeline-downloading-model-label = Downloading the transcription model
