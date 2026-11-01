# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor window: its title bar, tools, preview and menus.

editor-opening = Opening the editor
editor-screenshot-preview = Screenshot preview
editor-copy = Copy
# The title bar's command in a recording: copies the frame under the playhead, as the preview draws it.
editor-copy-frame = Copy Frame
editor-export = Export
editor-delete = Delete
editor-confirm-delete = Confirm deleting

## The tool column

editor-toolbar-more-tools = More tools
editor-toolbar-more-annotation-tools = More annotation tools
# Names the editor's tool column, Select and the pens, for assistive technology.
editor-toolbar-tools = Tools
# Names the editor's modes in its tool column, each opening an island for the whole recording, for assistive technology.
editor-toolbar-modes = Modes
# The mode that opens the script on its own, to cut the recording by its words.
editor-toolbar-transcript = Transcript
editor-tool-select = Select
editor-tool-select-name = Select screenshot
editor-tool-select-clip-name = Select recording clip
# The tool that crops the layer pressed: the screen or the camera of a recording, or a screenshot.
editor-tool-crop = Crop
# Names the Crop tool for assistive technology.
editor-tool-crop-name = Crop a layer
editor-tool-frame = Frame

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

## Scenes, as the lane and the Scene panel name them

editor-scene-custom = Custom
editor-scene-camera-only = Camera only
editor-scene-picture-in-picture = Picture in picture
editor-scene-screen-only = Screen only
editor-scene-side-by-side = Side by side
editor-scene-stacked = Stacked

## Annotations on the timeline lane, named by kind and place in the lane

editor-lane-arrow = Arrow { $number }
editor-lane-counter = Counter { $number }
editor-lane-drawing = Drawing { $number }
editor-lane-focus = Focus { $number }
editor-lane-highlight = Highlight { $number }
editor-lane-image = Image { $number }
editor-lane-magnifier = Magnifier { $number }
editor-lane-redaction = Redaction { $number }
editor-lane-shape = Shape { $number }
editor-lane-spotlight = Spotlight { $number }
editor-lane-text = Text { $number }

## annotation.

editor-pin-pin = Pin to Content
editor-pin-unpin = Unpin
editor-pin-hide = Content Out of View
editor-pin-show = Content Back in View
editor-pin-clear-corrections = Clear Corrections
editor-pin-tracking-section = Tracking

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
