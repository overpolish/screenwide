# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor's island: what is in hand, with its actions and choices.

# Names the island for assistive technology.
editor-island-label = Selection

## A lull on the strip: a stretch with nothing in it worth watching at full speed, whether the speaker stopped, a page was loading or the pointer sat idle.

editor-island-lull = Lull
editor-island-lull-cut = Cut the lull
editor-island-lull-confirm-cut = Confirm cutting
# Names the list of ways the lull can play, for assistive technology.
editor-island-lull-plays = How it plays
# The lull plays faster along with every other lull.
editor-island-lull-squeeze = Squeeze with the rest
editor-island-lull-keep = Keep as recorded
# The lull has a speed of its own, set by dragging its end. $speed is how many times as fast as recorded, such as 2.5.
editor-island-lull-retimed-faster = Retimed, { $speed }× faster
# As above, for a lull slowed down. $speed is less than 1, such as 0.5.
editor-island-lull-retimed-slower = Retimed, { $speed }× speed
# Shortens the lull to the length of a breath.
editor-island-lull-trim-to-breath = Trim to a breath
# The lull's note: dragging its end retimes it, and the key held through that drag trims it instead. $key is the key, drawn as a keycap: Option on a Mac, Alt on Windows.
editor-island-lull-hint = Hold { $key } and drag to trim.

## A range drawn across the strip

editor-island-range = Range
# Cuts the range and the time it takes.
editor-island-range-cut = Cut the range
editor-island-range-confirm-cut = Confirm cutting

## A beat on the strip: something that happens in the recording, said as it happens there

# A click on a control. $label is the control's name, such as a button's.
editor-island-beat-click = Click “{ $label }”
# Typing. $label is what was typed.
editor-island-beat-type = Type “{ $label }”
# Keys pressed together. $keys is the keys, drawn as keycaps.
editor-island-beat-press = Press { $keys }
# Switching apps. $app is the app's name.
editor-island-beat-switch = Switch to { $app }
# The picture changes to something new, such as another page.
editor-island-beat-screen = New screen
# The sound jumps well above its level: a shot, a crash, a cheer.
editor-island-beat-sound = Loud moment
# A line said in the recording, quoted. $line is the line.
editor-island-beat-speech = “{ $line }”
# Cuts the beat and the time it takes.
editor-island-beat-cut = Cut the beat
editor-island-beat-confirm-cut = Confirm cutting

## A moment flagged while recording, with the voice note recorded with it

# Cuts the moment and the time it takes.
editor-island-moment-cut = Cut the moment
editor-island-moment-confirm-cut = Confirm cutting
editor-island-moment-play = Play voice note
editor-island-moment-pause = Pause voice note
# Names the note's waveform for assistive technology. $moment is the moment's kind, such as Notable.
editor-island-moment-note = { $moment } voice note
editor-island-moment-no-speech = Nothing was said in this note
editor-island-moment-transcribe-failed = Could not transcribe this note
editor-island-moment-transcribing = Transcribing
editor-island-moment-waiting = Waiting to transcribe
# No transcription model is on this computer yet, so the note cannot be transcribed.
editor-island-moment-no-model = No model downloaded
# Downloads the transcription model.
editor-island-moment-download = Download
editor-island-moment-downloading = Downloading model
# Names the download's progress bar for assistive technology.
editor-island-moment-downloading-label = Downloading the transcription model

## A scene on the strip: how a stretch of the recording is framed

editor-island-scene = Scene
# Names the choice of which group of the scene's options shows beneath its row, for assistive technology.
editor-island-scene-groups = Scene options
# The groups of the scene's options, one shown beneath its row at a time.
editor-island-scene-layout = Layout
editor-island-scene-camera = Camera
editor-island-scene-screen = Screen
# Put one of a pane's figures back: its zoom to the whole picture, its position to the middle, its radius to the recording's own.
editor-island-scene-reset-zoom = Reset zoom
editor-island-scene-reset-radius = Reset radius
editor-island-scene-reset-position = Reset position
editor-island-scene-delete = Delete the scene
editor-island-scene-confirm-delete = Confirm deleting
# The scene's layout places the camera, which is left out of the video when the camera is exported as its own file; choosing a layout without the camera brings the scene back.
editor-island-scene-paused = This scene requires the camera.
# Names the choice of layouts for assistive technology.
editor-island-scene-layouts = Layouts
# A layout of the user's own, made by moving the panes by hand on the picture.
editor-island-scene-custom = Custom
editor-island-scene-side-by-side = Side by side
editor-island-scene-stacked = Stacked
editor-island-scene-picture-in-picture = Picture in picture
editor-island-scene-camera-only = Camera only
editor-island-scene-screen-only = Screen only
# Where the camera sits against the screen in a pair: on one side of the video or the other, or above or below.
editor-island-scene-camera-place = Camera
editor-island-scene-left = Left
editor-island-scene-right = Right
editor-island-scene-top = Top
editor-island-scene-bottom = Bottom
# How large the camera is against the screen in a pair.
editor-island-scene-camera-size = Camera size
# Trades a custom layout's screen and camera, each taking the other's place.
editor-island-scene-swap = Swap
# The corner of the screen the camera sits over.
editor-island-scene-corner = Corner
editor-island-scene-top-left = Top left
editor-island-scene-top-right = Top right
editor-island-scene-bottom-left = Bottom left
editor-island-scene-bottom-right = Bottom right
editor-island-scene-size = Size
editor-island-scene-small = Small
editor-island-scene-large = Large
# Saves a custom layout to use again in any recording; it then sits with the layouts.
editor-island-scene-save-layout = Save layout
editor-island-scene-remove-layout = Remove layout
editor-island-scene-confirm-remove-layout = Confirm removing
# A saved layout's tile. Saved layouts have no names of their own, so they are numbered in the order they sit.
editor-island-scene-saved-layout = Saved { $number }
# Keeps the camera on the speaker's face as they move.
editor-island-scene-follow-face = Follow face
# Names the spinner beside Follow face for assistive technology, while the camera looks for the speaker's face.
editor-island-scene-finding-face = Finding your face
# Puts the followed face back in the middle of the camera's box.
editor-island-scene-recenter = Recenter
editor-island-scene-zoom = Zoom
# How round the pane's corners are.
editor-island-scene-radius = Radius
# Which part of the pane's picture sits in the middle of its box.
editor-island-scene-position = Position
editor-island-scene-position-x = Horizontal position
editor-island-scene-position-y = Vertical position
# The axes the two position fields are for.
editor-island-scene-x = X
editor-island-scene-y = Y
# The screen turned in depth.
editor-island-scene-rotation = Rotation
editor-island-scene-rotation-x = X rotation
editor-island-scene-rotation-y = Y rotation
editor-island-scene-rotation-z = Z rotation
editor-island-scene-reset-rotation = Reset rotation
# Hangs under the row while no scene is in hand. The scene line runs along the timeline strip.
editor-island-scene-hint = Drag along the scene line to add a scene.
editor-island-scene-auto-zoom = Auto zoom
editor-island-scene-clear-auto-zooms = Clear all
editor-island-scene-confirm-clear-auto-zooms = Confirm clearing
# Plans the zooms again, replacing the planned zooms.
editor-island-scene-regenerate-auto-zooms = Regenerate
