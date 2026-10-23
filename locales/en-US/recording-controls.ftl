# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The recording controls in the island: setting up a capture, and the
### capture while it runs.

## What to record

recording-controls-mode = Recording type
recording-controls-mode-screen = Screen
recording-controls-mode-window = Window
recording-controls-mode-region = Region

## Capture buttons

recording-controls-screenshot = Screenshot
recording-controls-screenshot-options = Screenshot options
recording-controls-save-screenshot = Save Screenshot
recording-controls-copy-screenshot = Copy to Clipboard
recording-controls-delayed-screenshot = Delayed Screenshot
recording-controls-scrolling-screenshot = Capture Scrolling Region
recording-controls-record = Record
recording-controls-open-permissions = Open permissions
# The island's switch for the replay buffer, in its recording options.
recording-controls-island-replay = Replay buffer

## Inputs: camera, microphone and system audio

recording-controls-camera = Camera
recording-controls-microphone = Microphone
recording-controls-system-audio = System Audio
recording-controls-microphone-level = Microphone level
recording-controls-system-audio-level = System audio level
recording-controls-flip = Flip Horizontally
recording-controls-camera-missing = This camera is no longer connected
recording-controls-camera-mode-missing = This camera no longer offers { $size } at { $fps } fps
recording-controls-microphone-missing = This microphone is no longer connected
recording-controls-app-missing = A selected app is no longer running

## While recording

recording-controls-starting = Starting
recording-controls-finishing = Finishing
recording-controls-starting-label = Starting recording
recording-controls-finishing-label = Finishing recording
recording-controls-cancel-countdown = Cancel recording countdown
recording-controls-cancel-starting = Cancel starting recording
recording-controls-camera-preview = Camera confidence preview
recording-controls-pause = Pause recording
recording-controls-resume = Resume recording
recording-controls-stop = Stop recording
recording-controls-discard = Discard recording
recording-controls-confirm-discard = Confirm discarding
recording-controls-measuring-size = Measuring recording size
# $moment is the name of a kind of moment: a point flagged while recording.
recording-controls-recording-note = Recording a { $moment } voice note
recording-controls-moment-placed = { $moment } moment placed
recording-controls-note-no-microphone = No microphone for a voice note
recording-controls-note-no-microphone-short = No mic

## Island: the recording controls as one shape at the top of the screen that
## grows into whatever is being chosen and shows the recording while it runs.

recording-controls-island = Recording controls
recording-controls-island-source = Choose what to record
recording-controls-island-pick-window = Click a window to record it.
recording-controls-island-pick-region = Select a region of the screen to record.
# Beside the window picked to be recorded; picks another on the screen.
recording-controls-island-change-window = Change Window
# Beside the region selected to be recorded; selects another on the screen.
# The size of the area to be recorded, in pixels.
recording-controls-island-region-size = { $width } × { $height }
# Under the area's size, when it lies on more than one display.
recording-controls-island-region-displays =
    { $count ->
        [one] On { $count } display
       *[other] Across { $count } displays
    }
# The island's mode that records no screen, only the inputs that are on.
recording-controls-island-mode-none = No Screen
# In the No Screen mode, what will be recorded, from the inputs that are on;
# with none on, what to do instead.
recording-controls-island-no-screen =
    { $inputs ->
        [camera] Only your camera is recorded.
        [microphone] Only your microphone is recorded.
        [system-audio] Only your system audio is recorded.
        [camera-microphone] Your camera and microphone are recorded.
        [camera-system-audio] Your camera and system audio are recorded.
        [microphone-system-audio] Your microphone and system audio are recorded.
        [all] Your camera, microphone and system audio are recorded.
       *[none] Switch on an input to record without the screen.
    }
recording-controls-island-input-options = { $input } options
recording-controls-island-camera-locked = Screenwide can’t use your camera.
recording-controls-island-microphone-locked = Screenwide can’t use your microphone.
recording-controls-island-screen-locked = Screenwide needs permission to record your screen.
recording-controls-island-choose-another = Choose another
recording-controls-island-elapsed = Recording for { $time }
recording-controls-island-paused = Paused at { $time }
recording-controls-island-choose-window = Choose a Window
# Stops picking a window by clicking it, keeping the one chosen before.
recording-controls-island-cancel-pick = Cancel
recording-controls-island-record-options = Recording options
