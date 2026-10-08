# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The Settings window. Descriptions under a setting stay short: ten words
### or fewer where possible.

settings-logo = { -app-name }
settings-sections = Settings sections
settings-add-moment = Add Moment
settings-shortcuts = Shortcuts
settings-recording = Recording
settings-screenshots = Screenshots

## Sidebar sections, also the window's title

settings-section-general = General
settings-section-glide = Glide
settings-section-ruler = Ruler
settings-section-annotate = Annotate
settings-section-ocr = OCR
settings-section-moments = Moments
settings-section-transcription = Transcription
settings-section-shortcuts = Shortcuts

## General

settings-software-update = Software Update
settings-app-name = { -app-name }
settings-version = Version { $version }
settings-update-to = Update to v{ $version }
settings-check-updates = Check for Updates
settings-appearance = Appearance
settings-accent = Accent colour
settings-accent-description = Controls and highlights use the system accent colour, or { -app-name }'s own.
settings-accent-system = System
settings-accent-app = { -app-name }
settings-saving = Saving
settings-default-folder = Default folder
settings-projects-folder = Projects folder
settings-projects-folder-description = New recordings and their edits are kept here.
settings-projects-folder-reset = Use the default project folder
settings-recording-folder = Recording folder
settings-recording-folder-reset = Use the default recording folder
settings-screenshot-folder = Screenshot folder
settings-screenshot-folder-reset = Use the default screenshot folder
settings-export-folder-description = You can choose a different folder when saving.
settings-confidence-checks = Show camera and sound while recording
settings-confidence-checks-description = Check your camera, microphone and computer sound are working.
settings-auto-zoom = Zoom in automatically
settings-auto-zoom-description = Zoom in where you click and type.
settings-countdown = Recording countdown
settings-countdown-off = Off
settings-frame-rate = Frame rate
settings-frame-rate-description = More frames look smoother and make a larger file.
settings-fps = { $fps } fps
settings-capture = Capture
settings-include-app = Include { -app-name } in captures
settings-include-app-description = Show { -app-name }'s own windows in recordings and screenshots.
settings-screenshot-delay = Delayed Screenshot
settings-startup = Startup
settings-launch-at-login = Start when you sign in
settings-show-bar-on-launch = Show recording controls on startup

## Glide

settings-glide-enabled = Use Glide
settings-glide-enabled-description = Move and resize windows to fit your screen.
settings-glide-controls = Controls
settings-glide-mouse-modifier = Glide control for mouse
settings-glide-mouse-modifier-description = Hold while moving from a window's top bar.
settings-glide-thirds-modifier = Control for screen thirds
settings-glide-thirds-modifier-description = Hold during Glide to use thirds instead of halves.
# $platform is macos or windows: macOS calls virtual desktops Spaces.
settings-glide-spaces-modifier =
    { $platform ->
        [macos] Modifier for Spaces
       *[other] Modifier for virtual desktops
    }
settings-glide-spaces-modifier-description =
    { $platform ->
        [macos] Hold while moving between Spaces.
       *[other] Hold while moving between virtual desktops.
    }
settings-glide-monitors-modifier = Modifier for Monitors
settings-glide-monitors-modifier-description = Hold while moving between monitors.
settings-glide-behavior = Behavior
settings-glide-window-gap = Space around windows
settings-glide-cursor-follows = Move pointer with window
settings-glide-haptics = Trackpad feedback
settings-glide-haptics-description = Feel a tap when the next move is ready.
settings-glide-double-tap = Double-tap or double-click to center
settings-glide-double-tap-description = Two fingers, or hold mouse control while double-clicking.

## Ruler

settings-ruler-enabled = Use Ruler
settings-ruler-enabled-description = Measure sizes and distances on your screen.
settings-ruler-show = Show ruler
settings-ruler-crosshair = Toggle crosshair
settings-ruler-copy-colour = Copy colour
settings-ruler-copy-colour-description = Copies the colour under the pointer as a hex code.
settings-ruler-delete = Delete measurement
settings-ruler-delete-description = Hover a label or line; otherwise deletes the latest measurement.
settings-ruler-copy = Copy measurement
settings-ruler-copy-description = Copies the latest measurement, regardless of what you hover.
settings-ruler-undo = Undo
settings-ruler-redo = Redo
settings-ruler-stamp-horizontal = Horizontal stamp
settings-ruler-stamp-vertical = Vertical stamp
settings-ruler-stamp-description = Press to stamp; or hold, move, and release.
settings-ruler-guide-vertical = Vertical guide
settings-ruler-guide-vertical-description = Hold and click to place vertical guides.
settings-ruler-guide-horizontal = Horizontal guide
settings-ruler-guide-horizontal-description = Hold and click to place horizontal guides.
settings-ruler-tolerance = Cycle tolerance
settings-ruler-tolerance-description = Switch between clear, balanced, and subtle edge detection.
settings-ruler-radius = Measure radius
settings-ruler-radius-description = Hold over a corner to measure its radius.
settings-ruler-centerlines = Toggle centerlines

## Annotate

settings-annotate-enabled = Use Annotate
settings-annotate-enabled-description = Draw on your screen, and keep the annotations in a recording.
settings-annotate-draw = Draw on the screen
settings-annotate-keep = Keep annotations after exiting
settings-annotate-keep-description = They stay on screen until you clear them.
settings-annotate-clear = Clear annotations

## OCR: reading text and QR codes on screen

settings-ocr-enabled = Use OCR
settings-ocr-enabled-description = Select text or a QR code on your screen.
settings-ocr-activate = Read text or a QR code
settings-ocr-select-all = Select all text
settings-ocr-select-all-description = Select all recognized text.
settings-ocr-copy = Copy text
settings-ocr-copy-description = Copy selected text, or everything when nothing is selected.

## Moments: points flagged while recording, each kind with a colour and a
## shortcut. $moment is a kind's name, which the user chooses.

settings-new-moment = New Moment
settings-moment-name = { $moment } name
settings-moment-shortcut = { $moment } shortcut
settings-moment-color = { $moment } colour
settings-moment-remove = Remove { $moment }
settings-moment-confirm-remove = Confirm removing { $moment }
settings-moments-footnote = Press a moment's shortcut while recording to place it.
settings-voice-notes = Voice Notes
settings-voice-notes-enabled = Voice notes
settings-voice-notes-enabled-description = Hold a moment's shortcut to record a note.
settings-voice-notes-microphone = Microphone
settings-voice-notes-microphone-description = Same as Recording falls back to the system default.
settings-voice-notes-same-microphone = Same as Recording
settings-voice-notes-transcription = Transcription
# $model is a transcription model's name; $size its download size.
settings-voice-notes-needs-model = Needs the { $model } model ({ $size }).
settings-voice-notes-microphone-denied = { -app-name } needs microphone access for voice notes.

## Transcription. $model is a transcription model's name.

settings-transcription-local = Transcription runs on this computer. Audio is never uploaded.
settings-transcription-models = Models
settings-transcription-language = Language
settings-transcription-language-description = The language your recordings are spoken in.
# $language is the computer's own language.
settings-transcription-system-language = System ({ $language })
settings-transcription-detect = Detect Automatically
settings-transcription-download = Download
settings-transcription-cancel = Cancel
settings-transcription-downloading = Downloading the { $model } model
settings-transcription-remove = Remove the { $model } model
settings-transcription-confirm-remove = Confirm removing the { $model } model

## Keyboard shortcuts

settings-shortcut-toggle-bar = Show or hide recording controls
settings-shortcut-start-stop = Start or stop recording
settings-shortcut-pause-resume = Pause or resume recording
settings-shortcut-save-replay = Save replay
settings-shortcut-save-replay-description = Works while the replay buffer is on.
settings-shortcut-screenshot = Take a screenshot
settings-shortcut-screenshot-description = Choose an area of your screen to capture.
settings-shortcut-copy-screenshot = Copy a screenshot
settings-shortcut-copy-screenshot-description = Choose an area to copy, ready to paste elsewhere.
