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
# The transport's toggle that turns a drag across the strip into a stretch to cut; holding Option (Alt on Windows) does the same for one drag.
editor-timeline-cut = Cut
editor-timeline-cut-name = Cut a stretch of the recording
editor-timeline-copy-frame = Copy Frame
editor-timeline-copy-frame-label = Copy current frame
editor-timeline-preparing-audio = Preparing audio tracks
editor-timeline-preparing-audio-label = Preparing audio preview
editor-timeline-audio-visualizer = Audio visualizer
editor-timeline-segment = Timeline segment { $number }
editor-timeline-system-audio = System audio
editor-timeline-microphone = Microphone
# The meter beside the audio lanes, showing the level under the playhead.
editor-timeline-level = Audio level
# An audio track the recording did not name. $number counts from 1.
editor-timeline-audio-track = Audio { $number }

## Tools and zoom

editor-timeline-blade = Blade
editor-timeline-blade-name = Blade tool
editor-timeline-range = Range
editor-timeline-range-name = Range tool
editor-timeline-snap = Snap
editor-timeline-snap-name = Snap to edges
# Puts every clip back in the order it was recorded, after clips were moved.
editor-timeline-restore-order = Restore original order
editor-timeline-restore-order-confirm = Confirm restoring the order
editor-timeline-zoom-out = Zoom out
editor-timeline-zoom-out-label = Zoom timeline out
editor-timeline-zoom-in = Zoom in
editor-timeline-zoom-in-label = Zoom timeline in
editor-timeline-fit = Fit timeline
# A part of the recording that was cut out, put back by pressing its marker
# over the ruler. $duration is its length, such as 2.4 sec.
editor-timeline-restore-cut = Restore { $duration } cut
editor-timeline-restore-cut-at = Restore the { $duration } cut at { $time }

## Tracks. $track names a track.

editor-timeline-shortcuts = Shortcuts
editor-timeline-captions = Captions
editor-timeline-annotations = Annotations
editor-timeline-scenes = Scenes
editor-timeline-camera-separate = Saved as its own file
# A stretch of the camera, drawn above it on the timeline, where the background behind the person is taken away. $background is what is shown instead: none, blur or picture.
editor-timeline-camera-background-range =
    { $background ->
        [blur] Background blurred
        [picture] Background picture
       *[none] Background removed
    }
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
# The window on the overview round the part of the recording the strip shows.
editor-timeline-view = Part in view
# The overview's end, dragged in to play every lull faster.
editor-timeline-squeeze = Squeeze the lulls
# A mark where the recording no longer plays in the order it was recorded, after a stretch was moved; pressed, it puts the stretch back.
editor-timeline-put-back = Put back where it was recorded
# A shortcut pressed while recording, drawn over the picture where it was pressed. $keys are its keys, joined with +, such as Command+S.
editor-timeline-shortcut = Shortcut { $keys }
# A mark where the recording is split, so something moved can land there; pressed, it takes the split away.
editor-timeline-remove-split = Remove the split
