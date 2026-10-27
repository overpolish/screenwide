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
