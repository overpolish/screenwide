# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The recording bar, which sets up a capture, and the dock shown while
### recording.

recording-controls-cancel = Cancel

## What to record

recording-controls-mode = Recording type
recording-controls-mode-screen = Screen
recording-controls-mode-window = Window
recording-controls-mode-region = Region
recording-controls-mode-camera = Camera
recording-controls-mode-audio = Audio

## Capture buttons

recording-controls-screenshot = Screenshot
recording-controls-screenshot-options = Screenshot options
recording-controls-save-screenshot = Save Screenshot
recording-controls-copy-screenshot = Copy to Clipboard
recording-controls-delayed-screenshot = Delayed Screenshot
recording-controls-scrolling-screenshot = Capture Scrolling Region
recording-controls-record = Record
recording-controls-open-permissions = Open permissions
# The replay buffer keeps recording in the background so the last moments can
# be saved after they happen.
recording-controls-replay =
    { $seconds ->
        [one] Replay buffer, keeps the last { $seconds } second
       *[other] Replay buffer, keeps the last { $seconds } seconds
    }

## Inputs: camera, microphone and system audio

recording-controls-camera = Camera
recording-controls-microphone = Microphone
recording-controls-system-audio = System Audio
recording-controls-system-audio-picker = System audio
recording-controls-choose-camera = Choose camera
recording-controls-choose-microphone = Choose microphone
recording-controls-choose-system-audio = Choose system audio
recording-controls-no-camera = No Camera
recording-controls-no-microphone = No Microphone
# Several apps chosen to record the sound of.
recording-controls-sources =
    { $count ->
        [one] { $count } Source
       *[other] { $count } Sources
    }
recording-controls-off = Off
recording-controls-resolution = Resolution
recording-controls-options = Options
recording-controls-flip = Flip Horizontally
# Records at 50 frames a second, so 50 Hz mains lighting does not flicker.
recording-controls-anti-flicker = Anti-Flicker 50 Hz
# $size is a camera's frame size, such as "1920 × 1080"; $fps its frame rate.
recording-controls-camera-mode = { $size } · { $fps } fps
recording-controls-camera-missing = This camera is no longer connected
recording-controls-camera-mode-missing = This camera no longer offers { $size } at { $fps } fps
recording-controls-microphone-missing = This microphone is no longer connected
recording-controls-app-missing = A selected app is no longer running

## The dock shown while recording

recording-controls-starting = Starting
recording-controls-finishing = Finishing
recording-controls-starting-label = Starting recording
recording-controls-finishing-label = Finishing recording
recording-controls-cancel-countdown = Cancel recording countdown
recording-controls-cancel-starting = Cancel starting recording
recording-controls-cancel-finishing = Cancel finishing recording
recording-controls-camera-preview = Camera confidence preview
recording-controls-pause = Pause recording
recording-controls-resume = Resume recording
recording-controls-stop = Stop recording
recording-controls-discard = Discard recording
recording-controls-confirm-discard = Confirm discarding
# $moment is the name of a kind of moment: a point flagged while recording.
recording-controls-recording-note = Recording a { $moment } voice note
recording-controls-moment-placed = { $moment } moment placed
recording-controls-note-no-microphone = No microphone for a voice note
recording-controls-note-no-microphone-short = No mic
