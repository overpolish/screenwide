# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor's tool panels: the controls each tool shows while it is in
### hand. Row titles are short; most are one or two words.

editor-panels-nothing-selected = Nothing selected
editor-panels-size = Size
editor-panels-position = Position
editor-panels-reset-position = Reset position
editor-panels-inset = Inset
editor-panels-drop-shadow = Drop shadow
editor-panels-reset = Reset
editor-panels-reset-all = Reset all
editor-panels-recenter = Recenter
editor-panels-delete = Delete
editor-panels-clear-all = Clear all
editor-panels-apply-to-all = Apply to all
editor-panels-camera = Camera
editor-panels-screen = Screen
editor-panels-screenshot = Screenshot
editor-panels-audio = Audio
# A layer of a screenshot, counted from the bottom of the stack.
editor-panels-layer = Layer { $number }
editor-panels-crop = Crop

## Annotations

editor-panels-annotations-selected =
    { $count ->
        [one] { $count } annotation selected
       *[other] { $count } annotations selected
    }
editor-panels-delete-annotations = Delete the selected annotations
editor-panels-reverse = Reverse
editor-panels-reverse-arrow = Reverse the arrow
editor-panels-clear-drawings = Clear all drawings
editor-panels-soft-edge = Soft edge
editor-panels-animate = Animate
editor-panels-redaction-style = Style
editor-panels-strength = Strength
editor-panels-redaction-note = Use Erase or Colour for sensitive content.
editor-panels-remove-color = Remove Colour
editor-panels-color-actions = Colour actions

## Images placed as annotations

editor-panels-image-description = Paste or drag images in
editor-panels-choose = Choose
editor-panels-choose-image = Choose an image
editor-panels-replace = Replace
editor-panels-replace-image = Replace the image
editor-panels-flip = Flip
editor-panels-mirror-image = Mirror the image
# Which frame of a moving image shows; in a recording, the one it starts on.
editor-panels-frame = Frame
editor-panels-start-frame = Start frame
editor-panels-playback = Playback
editor-panels-playback-loop = Loop
editor-panels-playback-once = Once
# A slight, slow turn and drift that keeps a placed image alive.
editor-panels-sway = Sway

## Cursor

editor-panels-cursor = Cursor
editor-panels-cursor-name = Cursor effects
editor-panels-cursor-none = This recording has no cursor movement.
editor-panels-cursor-size = Cursor size
editor-panels-cursor-clip = Keep cursor inside the recording
editor-panels-cursor-smooth = Smooth movement
editor-panels-cursor-motion-blur = Motion blur
editor-panels-cursor-click-animation = Click animation

## Keyboard shortcuts drawn over a recording

editor-panels-keyboard = Keyboard
editor-panels-keyboard-name = Keyboard shortcuts
editor-panels-keyboard-empty = This recording has no keyboard shortcuts.
editor-panels-keyboard-show = Show shortcuts
editor-panels-shortcut-x = Shortcut X position
editor-panels-shortcut-y = Shortcut Y position
editor-panels-keyboard-animation = Animation
editor-panels-keyboard-pop = Pop
editor-panels-keyboard-fade = Fade
editor-panels-keyboard-none = None
editor-panels-keyboard-appearance = Appearance
editor-panels-keyboard-dark = Dark
editor-panels-keyboard-light = Light
editor-panels-keyboard-restore = Restore all shortcuts

## Frame and background

editor-panels-remove-preset = Remove Preset
editor-panels-background-actions = Background actions

## Scenes: how the screen and camera are laid out over time

editor-panels-scene = Scene
editor-panels-scene-needs-screen = Scenes need a recording with a screen.
editor-panels-scene-paused = This scene is paused while the camera is left out.
editor-panels-output = Output
editor-panels-output-combined = One video
editor-panels-output-separate = Separate files
editor-panels-swap = Swap
editor-panels-zoom = Zoom
# $pane is the part of the scene being framed: Screen or Camera.
editor-panels-pane-zoom = { $pane } zoom
editor-panels-pane-radius = { $pane } radius
editor-panels-pane-x = { $pane } X position
editor-panels-pane-y = { $pane } Y position
editor-panels-auto-zoom = Auto zoom
editor-panels-regenerate = Regenerate
editor-panels-clear-auto-zooms = Clear all auto zooms
editor-panels-save-template = Save template
editor-panels-remove-template = Remove Template
editor-panels-template-actions = Template actions
editor-panels-camera-size = Camera size
editor-panels-one-third = One third
editor-panels-two-thirds = Two thirds
editor-panels-corner = Corner
editor-panels-corner-top-left = Top left
editor-panels-corner-top-right = Top right
editor-panels-corner-bottom-left = Bottom left
editor-panels-corner-bottom-right = Bottom right
editor-panels-size-small = Small
editor-panels-size-large = Large

## Audio

editor-panels-volume = Volume
editor-panels-reset-volume = Reset volume
editor-panels-reduce-noise = Reduce noise
editor-panels-noise-cleaning = Removing noise
editor-panels-silences = Silences
editor-panels-remove = Remove
editor-panels-restore-all = Restore all
editor-panels-restore-silences = Restore all silences
editor-panels-silences-finding = Finding pauses
editor-panels-silences-none = No long pauses found
# $duration is how much shorter the cuts make the recording, such as 00:48.
editor-panels-silences-removed =
    { $count ->
        [one] One cut, { $duration } shorter
       *[other] { $count } cut, { $duration } shorter
    }
