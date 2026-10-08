# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor window: its title bar, tools, preview and menus.

editor-logo = { -app-name }
editor-opening = Opening the editor
editor-screenshot-preview = Screenshot preview
editor-zoom = Zoom
editor-copy = Copy
editor-export = Export
editor-delete = Delete
editor-confirm-delete = Confirm deleting

## Tools in the title bar

editor-pointer-tools = Pointer tools
editor-canvas-tools = Canvas tools
editor-toolbar-more-tools = More tools
editor-toolbar-more-annotation-tools = More annotation tools
editor-tool-select = Select
editor-tool-select-name = Select screenshot
editor-tool-select-clip-name = Select recording clip
editor-tool-marquee = Marquee
editor-tool-marquee-name = Choose annotations with a marquee
editor-tool-canvas = Resize canvas
editor-tool-frame = Frame
editor-tool-frame-name = Resize recording frame
editor-tool-crop = Crop
editor-tool-crop-name = Crop screenshot
editor-tool-crop-clip-name = Crop recording clip

## Menus

editor-annotation-actions = Annotation actions
editor-layer-actions = Layer actions
# Moves through the drawing order, as video layers are moved.
editor-arrange = Arrange
editor-arrange-forward = Move Forward
editor-arrange-front = Move to Front
editor-arrange-backward = Move Backward
editor-arrange-back = Move to Back

## The recording preview

editor-recording-preview = Recording preview
editor-recording-composed-preview = Composed recording preview
editor-recording-screen-preview = Screen preview
editor-recording-camera-preview = Camera preview
editor-recording-preparing-preview = Preparing the preview
editor-recording-preparing-recording = Preparing recording preview
editor-recording-effects = Effects
editor-recording-shortcut = Shortcut

## Scenes, as the lane and the Scene panel name them

editor-scene-custom = Custom
editor-scene-zoom = Zoom
editor-scene-camera-only = Camera only
editor-scene-picture-in-picture = Picture in picture
editor-scene-screen-only = Screen only
editor-scene-side-by-side = Side by side
editor-scene-stacked = Stacked

## Annotations on the timeline lane, named by kind and place in the lane

editor-lane-arrow = Arrow { $number }
editor-lane-counter = Counter { $number }
editor-lane-drawing = Drawing { $number }
editor-lane-highlight = Highlight { $number }
editor-lane-image = Image { $number }
editor-lane-magnifier = Magnifier { $number }
editor-lane-redaction = Redaction { $number }
editor-lane-shape = Shape { $number }
editor-lane-spotlight = Spotlight { $number }
editor-lane-text = Text { $number }
# $clip names a clip on a timeline lane.
editor-clip-start = Start of { $clip }
editor-clip-end = End of { $clip }

## Annotations pinned to moving content in a recording. $clip names the
## annotation.

editor-pin-pin = Pin to Content
editor-pin-unpin = Unpin
editor-pin-hide = Content Out of View
editor-pin-show = Content Back in View
editor-pin-clear-corrections = Clear Corrections
editor-pin-tracking-section = Tracking
editor-pin-tracking = Tracking { $clip }
editor-pin-keyframe-actions = Keyframe actions
# A keyframe on a pinned clip's lane, by what it does. $clip names the clip.
editor-pin-keyframe =
    { $kind ->
        [correction] Correction of { $clip }
        [outOfView] Out of view of { $clip }
       *[pinned] Pinned frame of { $clip }
    }
editor-pin-delete-correction = Delete Correction
editor-pin-delete-out-of-view = Delete Out of View
editor-pin-delete-pinned-frame = Delete Pinned Frame
editor-pin-off-frame = Off the frame
editor-pin-under-cover = Under a cover
editor-pin-out-of-view-stretch = Out of view
editor-pin-lost = Lost here, check

## Time left on a running export

editor-eta-under-minute = Less than a minute remaining
# $duration is a list of the parts below, such as "1 hr 5 min".
editor-eta = About { $duration } remaining
editor-eta-days =
    { $count ->
        [one] { $count } day
       *[other] { $count } days
    }
editor-eta-hours = { $count } hr
editor-eta-minutes = { $count } min
