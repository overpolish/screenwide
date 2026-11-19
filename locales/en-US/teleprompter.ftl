# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: Elastic-2.0

### Teleprompter: a script written and read in a floating window under the camera, moving at a steady pace or following the voice. Each script is a project of its own.

# Names the button that starts the script moving.
teleprompter-play = Play
teleprompter-pause = Pause
# Names the button that goes back to the script's start.
teleprompter-restart = Back to the start
# Names the toggle that moves the script along as the reader speaks, rather than at a steady pace.
teleprompter-follow-voice = Follow voice
# Names the field for the steady pace, used while the voice is not followed.
teleprompter-pace = Pace
# The unit after the steady pace's figure: words read a minute.
teleprompter-pace-unit = wpm
teleprompter-text-size = Text size
# Names each of the two dashed lines at the text's sides, dragged to make the column of text narrower or wider.
teleprompter-column-width = Column width
# Names the toggle that flips the whole window left to right, for reading through the glass of a teleprompter rig.
teleprompter-mirror = Mirror
# Names the toggle that fills the display the teleprompter is on, such as a teleprompter's own monitor.
teleprompter-fullscreen = Fill the display
teleprompter-close = Close teleprompter
teleprompter-voice-loading = Getting ready to listen
teleprompter-voice-no-microphone = Screenwide may not use the microphone, so it moves at a steady pace.
teleprompter-voice-failed = Could not hear the voice, so it moves at a steady pace.
# Names the script's name in the title bar, which is renamed where it is shown, for assistive technology.
teleprompter-script-name = Script name
# Names the field the script is written in, for assistive technology.
teleprompter-script-label = Script
teleprompter-script-placeholder = Write what you will say. Leave an empty line between paragraphs.
teleprompter-save-failed = Could not save the script.

## Cues: a picture of a recording or a screenshot pinned to a word of the script, shown while the script is read.

# The item of the script's right-click menu that pins a cue to the word clicked.
teleprompter-cue-add = Add cue
# The item of a cue's menu that chooses another picture for it.
teleprompter-cue-replace = Replace cue
teleprompter-cue-remove = Remove cue
# Names the menu opened on a cue or on the script, for assistive technology.
teleprompter-cue-menu = Cue
# Names a cue's mark beside its word. $title is the recording's or screenshot's name.
teleprompter-cue-mark = Cue: { $title }
# Names the panel that lists the recordings and screenshots a cue can show.
teleprompter-cue-picker = Choose a recording or screenshot
teleprompter-cue-search = Search recordings and screenshots
# Names the list of recordings and screenshots, for assistive technology.
teleprompter-cue-sources = Recordings and screenshots
teleprompter-cue-none = No recordings or screenshots
# Names the button that goes back from a recording's moments to the list.
teleprompter-cue-back = Back
# Names the slider that picks the moment of the recording the cue shows.
teleprompter-cue-moment = Moment
teleprompter-cue-failed = Could not make the cue.
# Said under a screenshot's name in the cue picker, where a recording's length is said instead.
teleprompter-cue-screenshot = Screenshot
