# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The Export island: saving a finished recording or screenshot from over the editor's picture.

## Before the export: the file, its quality and options.

editor-export-title-recording = Export recording
editor-export-title-audio = Export audio
editor-export-title-screenshot = Export screenshot
# Names the field the file's name is typed in.
editor-export-file-name = File name
# Names the folder field the file is saved in.
editor-export-folder = Folder
# Names the choice of picture format.
editor-export-format = Format
editor-export-format-png = PNG
editor-export-format-jpeg = JPEG
# Names the four looks, which trade the file's size against its quality.
editor-export-quality = Quality
editor-export-look-smallest = Smallest
editor-export-look-smaller = Smaller
editor-export-look-balanced = Balanced
editor-export-look-original = Original
# A look's picture size, in pixels.
editor-export-dimensions = { $width } × { $height }
# Shown in place of a file size while it is being measured.
editor-export-measuring = Measuring
# A subtitle file made from the transcript, written beside the exported video.
editor-export-subtitles = Save subtitles
# Names the choice of subtitle file format, beside Save subtitles.
editor-export-subtitles-format = Subtitle format
editor-export-subtitles-srt = SRT
editor-export-subtitles-vtt = WebVTT
editor-export-camera-separate = Save the camera as its own file
# Names the colour that fills a separate camera file where its background is taken away, a colour another editor keys out.
editor-export-camera-key = Background colour
editor-export-camera-key-green = Green
editor-export-camera-key-blue = Blue
editor-export-mix-audio = Mix the audio tracks into one
# $platform is windows or macos: the file manager is Explorer or Finder.
editor-export-reveal-when-done =
    { $platform ->
        [windows] Show in Explorer when done
       *[other] Show in Finder when done
    }
# The project is moved to the app's Recently Deleted once the export is written.
editor-export-delete-after = Move to Recently Deleted after exporting
editor-export-action = Export
# $reason is the system's own explanation, in English.
editor-export-failed = The export failed. { $reason }
editor-export-try-again = Try again
# The picture as drawn beside it as the export writes it, compared across a divider.
editor-export-compare-original = Original
editor-export-compare-export = Export
# Names the divider dragged across the compared picture.
editor-export-compare-divider = Divider between original and export
editor-export-compare-making = Making the comparison
editor-export-compare-failed = The comparison could not be made.
# Under the compared picture when a screenshot saved as JPEG has see-through parts, such as rounded corners with no background behind them.
editor-export-jpeg-see-through = JPEG has no transparency, so see-through parts are saved as white.

## While the export runs.

editor-export-saving-audio = Saving audio
editor-export-saving-screenshot = Saving screenshot
editor-export-saving-camera = Saving camera
editor-export-saving-recording = Saving recording
editor-export-finalizing = Finalizing recording
# Shown under the work while the export has no figure yet.
editor-export-starting = Starting
editor-export-progress = Save progress
editor-export-cancel = Cancel export
# Pressed a second time, it stops the export and throws the partial file away.
editor-export-confirm-cancel = Stop and discard
editor-export-canceling = Canceling

## Once the file is written.

editor-export-done = Exported
# $platform is windows or macos: the file manager is Explorer or Finder.
editor-export-reveal =
    { $platform ->
        [windows] Show in Explorer
       *[other] Show in Finder
    }
editor-export-copy = Copy the file
# Names the written file, which is dragged out into another app. $name is the file's name.
editor-export-drag-file = Drag { $name } to another app
