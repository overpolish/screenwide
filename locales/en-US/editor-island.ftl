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

## The recording's sound, every track at once

# The island for the recording's sound: each track's volume, and the microphone's tools.
editor-island-sound = Sound
# Names a track's volume field for assistive technology. $track is the track's name, such as Microphone.
editor-island-sound-volume = { $track } volume
# Puts a track back at the level it was recorded at. $track is the track's name, such as Microphone.
editor-island-sound-reset-volume = Reset { $track } volume
# The library of songs and sound effects added to any recording, as the Library island lists them.
editor-island-sound-library = Music and sound effects
# The Library island's plus: chooses files of the kind shown (songs, sound effects or pictures) and keeps them in the library, without adding them to the timeline or the picture.
editor-island-library-add = Add to the library
# The choice at the top of the Library island between its songs, its sound effects and its pictures; for songs and sound effects it is also what one added becomes.
editor-island-sound-library-kind = Show
editor-island-sound-library-music = Music
editor-island-sound-library-effects = Sound effects
editor-island-library-images = Images
editor-island-sound-library-search = Search
editor-island-sound-library-empty-music = Songs you add are kept here, for any recording.
editor-island-sound-library-empty-effects = Sound effects you add are kept here, for any recording.
# Nothing in the library has the words searched for in its name.
editor-island-sound-library-no-match = Nothing matches
# A song or sound effect in the library, for assistive technology: pressing it adds it to the timeline. $title is its title, $length how long it plays.
editor-island-sound-library-add = Add { $title }, { $length }
# How long a sound under ten seconds plays, as the library lists it, in seconds to a tenth. $seconds is the number, already formatted.
editor-island-sound-library-seconds = { $seconds } s
# Plays a song or sound effect in the library, or stops it. $title is its title.
editor-island-sound-library-play = Play { $title }
editor-island-sound-library-stop = Stop { $title }
# A song or sound effect's waveform in the library, for assistive technology: pressing along it plays it from there. $title is its title.
editor-island-sound-library-play-from = Play { $title } from where you press
# Takes a song or sound effect out of the library; projects that use it keep their own copy. $title is its title.
editor-island-sound-remove = Remove { $title } from the library
# Confirms taking a song, sound effect or picture out of the library.
editor-island-library-confirm-remove = Confirm removing
editor-island-library-empty-images = Pictures you give images are kept here, for any recording or screenshot.
# A picture in the library, for assistive technology: pressing it places it as a new image. $name is the name it was given or came with, or when it was added.
editor-island-library-picture-add = Add { $name }
# Names a picture in the library that has no name yet, as pasted pictures do. $date is when it was added.
editor-island-library-picture-unnamed = Picture added { $date }
# Takes a picture out of the library; projects that show it keep their own copy. $name names it as above.
editor-island-library-picture-remove = Remove { $name } from the library
# The field a song's, sound effect's or picture's name is edited in, for assistive technology.
editor-island-library-name = Name
# Searches online (Openverse, a library of openly licensed works, which people need not know by name) for the words in the Library's search field, under its own results. $query is the words.
editor-island-openverse-search = Search online for “{ $query }”
# Heads what the online search found, under the library's own.
editor-island-openverse-heading = Found online
editor-island-openverse-searching = Searching online
# Heads the online search while its first results are on their way, before it becomes the heading above.
editor-island-openverse-searching-heading = Searching
editor-island-openverse-more = More results
editor-island-openverse-none = Nothing found online for that
editor-island-openverse-busy = There have been too many searches for now.
editor-island-openverse-unavailable = The online search is not answering right now.
# Each installation may search online a set number of times a day, so no one uses up the searches everyone shares. $time is when the limit resets, in the person's own time, already formatted, such as 01:00; $midnight is yes where that is midnight, said as a word rather than 00:00.
editor-island-openverse-used-up = You've reached today's search limit. { $midnight ->
    [yes] Try again after midnight.
   *[no] Try again after { $time }.
}
# Asks the online search again after it failed.
editor-island-openverse-retry = Try again
# For assistive technology, while a song, sound effect or picture found online downloads into the library. $title is its title.
editor-island-openverse-fetching = Downloading { $title }
# $title is what was found.
editor-island-openverse-add-failed = “{ $title }” could not be downloaded.
# Something found online was looked up again as it was downloaded, and is gone or no longer under a licence Screenwide may use, so it was not added. $title is what was found.
editor-island-openverse-withdrawn = “{ $title }” is no longer offered under a licence Screenwide can use, so it was not added.
# The button that downloads something found online into the library, for assistive technology. $title is its title, $creator who made it, $license its licence's short name.
editor-island-openverse-fetch-by = Download { $title } by { $creator } to the library, { $license }
editor-island-openverse-fetch = Download { $title } to the library, { $license }
# The licences what is found on Openverse may have: CC BY, which an export credits, CC0, and the public domain.
editor-island-openverse-license-by = CC BY
editor-island-openverse-license-cc0 = CC0
editor-island-openverse-license-pdm = Public domain
# The filters under the Library's search field while it shows music, which narrow the songs found online. Each shows what it is set to and opens its choices.
editor-island-music-filters = Music filters
# A music filter that is set, for assistive technology. $filter is the filter's name below, $choice what it is set to.
editor-island-music-filter-set = { $filter }: { $choice }
# How fast a song goes.
editor-island-music-filter-tempo = Tempo
editor-island-music-filter-tempo-any = Any tempo
editor-island-music-filter-tempo-slow = Slow
editor-island-music-filter-tempo-medium = Medium
editor-island-music-filter-tempo-fast = Fast
# Whether a song is sung.
editor-island-music-filter-vocals = Vocals
editor-island-music-filter-vocals-any = With or without vocals
editor-island-music-filter-vocals-instrumental = Instrumental
editor-island-music-filter-vocals-vocal = With vocals
# How a song feels.
editor-island-music-filter-mood = Mood
editor-island-music-filter-mood-any = Any mood
editor-island-music-filter-mood-happy = Happy
editor-island-music-filter-mood-uplifting = Uplifting
editor-island-music-filter-mood-energetic = Energetic
editor-island-music-filter-mood-calm = Calm
editor-island-music-filter-mood-dreamy = Dreamy
editor-island-music-filter-mood-romantic = Romantic
editor-island-music-filter-mood-sad = Sad
editor-island-music-filter-mood-dark = Dark
editor-island-music-filter-mood-epic = Epic

## A song or sound effect on the strip, laid under the recording

# Names the choice between the two kinds of clip, which sets how it plays and which lane it sits in.
editor-island-audio-kind = Kind
editor-island-audio-song = Song
editor-island-audio-effect = Sound effect
# Beside the Song and Sound effect choice for a few seconds after Song is pressed for a sound effect that would land on a song, and in the library's column while it shows music with no room left for another song: songs never overlap, and there is no room for it among them.
editor-island-audio-no-song-room = No room on the music track
editor-island-audio-volume = Volume
# Puts the clip back at the level it plays at by default: under the voice for a song, a little under it for a sound effect.
editor-island-audio-reset-volume = Reset volume
# The clip's stretch over its sound's waveform, dragged to choose where in the sound the clip starts playing.
editor-island-audio-plays-from = Plays from
# A sound effect's stretch over its whole sound's waveform: dragged to move it, its ends dragged to trim it.
editor-island-audio-plays = Plays
# What the sound effect's stretch says to assistive technology. $from is where in the sound it starts, $length how long it plays, both as durations.
editor-island-audio-plays-value = From { $from }, for { $length }
# Always under the clip's waveform, whose window is dragged to choose where the clip starts. $key is the key held, ⌘ on a Mac and Ctrl on Windows.
editor-island-audio-fine-hint = Hold { $key } while dragging to move it finely.
# Plays the clip from where it starts in its sound, in the island.
editor-island-audio-preview = Play the clip
editor-island-audio-stop-preview = Stop playing the clip
# A song played without its singing, which the music separation model takes out.
editor-island-audio-instrumental = Instrumental
# While the singing is being taken out of a song played as an instrumental.
editor-island-audio-removing-singing = Removing the singing
# Puts another song or sound effect in the clip's place, chosen from a file.
editor-island-audio-replace = Replace
editor-island-audio-delete = Delete the clip
# Takes the clip's song or sound effect into the library from the project's own copy, where the library no longer holds it, for assistive technology.
editor-island-audio-keep = Add the sound to the library
editor-island-audio-confirm-delete = Confirm deleting

## The recording's tracks, each held in the export and the preview or left out

# The island the transport's Tracks button brings out: a switch for each recorded track.
editor-island-tracks = Tracks
# The switch for every song laid under the recording, which keeps them in the export and the preview or leaves them out.
editor-island-tracks-music = Music
# The switch for every sound effect laid under the recording.
editor-island-tracks-effects = Sound effects

## The recording's shortcuts, every one at once

# Deletes the shortcuts chosen on the strip. $count is how many are chosen.
editor-island-keyboard-delete =
    { $count ->
        [one] Delete the shortcut
       *[other] Delete { $count } shortcuts
    }
editor-island-keyboard-confirm-delete = Confirm deleting

## The recording's cursor, for the whole recording

# Puts the cursor's size and effects back as they first were.
editor-island-cursor-reset = Reset cursor

## The recording's script, what was said as a whole

# The island for the recording's script, open beside the Captions island or on its own.
editor-island-script = Script
# Under the script while it is open with the captions: how to edit it. $key is drawn as a keycap: Command on macOS, Ctrl on Windows.
editor-island-script-captions-hint = Hold { $key } and click or drag to hide or show words. Right-click for breaks and key words.
# Takes back every edit made in the script: breaks, hidden or shown words and key words placed by hand, and spelling fixes, which go back to what was heard.
editor-island-script-reset = Reset caption edits
# The search at the end of the script's row: its placeholder and accessible name.
editor-island-script-search = Search script
# The toggle beside the search that opens Replace under it.
editor-island-script-replace = Replace
# The text field for what every match of the search is replaced with.
editor-island-script-replace-with = Replace with
# Replaces every match of the search, as one undo step.
editor-island-script-replace-all = Replace all
# Under the script on its own: how to edit it. $key is drawn as a keycap: Command on macOS, Ctrl on Windows.
editor-island-script-transcript-hint = Hold { $key } and click or drag to remove or restore words.
# Cuts the pauses long enough to cut. $count is how many it would cut, 0 before the recording has been listened through for them.
editor-island-script-remove-pauses =
    { $count ->
        [0] Remove pauses
        [one] Remove 1 pause
       *[other] Remove { $count } pauses
    }
# Cuts every filler word, such as "um". $count is how many there are.
editor-island-script-remove-fillers =
    { $count ->
        [one] Remove 1 filler
       *[other] Remove { $count } fillers
    }
# Cuts every word marked as profanity. $count is how many there are.
editor-island-script-remove-profanity =
    { $count ->
        [one] Remove 1 profane word
       *[other] Remove { $count } profane words
    }
# Shown on the chip that removes the pauses, after its icon. $count is how many there are.
editor-island-script-pause-count =
    { $count ->
        [one] 1 pause
       *[other] { $count } pauses
    }
# Shown on the chip that removes the filler words, after its icon. $count is how many there are.
editor-island-script-filler-count =
    { $count ->
        [one] 1 filler
       *[other] { $count } fillers
    }
# Shown on the chip that removes the words marked as profanity, after its icon. $count is how many there are.
editor-island-script-profanity-count =
    { $count ->
        [one] 1 profane word
       *[other] { $count } profane words
    }
# While the recording is listened through for the pauses to cut.
editor-island-script-finding-pauses = Finding pauses
# After Remove pauses found none long enough to cut.
editor-island-script-no-pauses = No long pauses found.
# Transcribes the recording again, replacing its words.
editor-island-script-transcribe-again = Transcribe again
# Brings back every word and pause cut in the script.
editor-island-script-restore-all = Restore all

## The recording's captions, all of them at once

# The island for the recording's captions: their look, their layout and what they say.
editor-island-captions = Captions
# A toggle in the island's row: whether the captions are drawn over the recording.
editor-island-captions-show = Show captions
# Names the row choosing the type size: a step from the template's own size.
editor-island-captions-size = Size
# Names the row choosing the key word's size, shown where the template sets one word apart.
editor-island-captions-accent-size = Key word size
editor-island-captions-size-small = Small
editor-island-captions-size-medium = Medium
editor-island-captions-size-large = Large
editor-island-captions-size-extra-large = Extra large
# A toggle: whether the captions are set in capitals whatever was said.
editor-island-captions-all-caps = All caps
# Names the grid of places on the picture the captions can sit.
editor-island-captions-position = Position
# How many lines one caption may take, where its words are set in lines.
editor-island-captions-lines = Lines
editor-island-captions-one-line = One line
editor-island-captions-two-lines = Two lines
# Names the field for the most words one caption holds, across its lines, for assistive technology.
editor-island-captions-max-words = Max words
# Before the most words one caption holds, as the field reads: Max 8 words.
editor-island-captions-max = Max
# After the most words one caption holds, as the field reads: Max 8 words. $count is the number.
editor-island-captions-words-unit =
    { $count ->
        [one] word
       *[other] words
    }
# A toggle: a caption stays up until the next one starts, through any silence short of a stretch with nothing said.
editor-island-captions-hold-until-next = Hold until next caption
editor-island-captions-hide-punctuation = Hide punctuation
editor-island-captions-hide-fillers = Hide filler words
# A toggle: whether a word found to be profanity is starred out of the captions.
editor-island-captions-censor-profanity = Censor profanity
# The filler word the Hide filler words toggle draws, struck through while it is on: the language's most common one, as short as it can be.
editor-island-captions-filler-sample = um
# Brings back the captions taken out of the video. $count is how many there are.
editor-island-captions-restore =
    { $count ->
        [one] Restore 1 caption
       *[other] Restore { $count } captions
    }
# Puts the captions' look, layout and rules back as last set as the default.
editor-island-captions-reset = Reset to default
# Makes new captions start looking, laid out and saying things as these do.
editor-island-captions-set-default = Set as default

## Several annotations chosen at once, with the marquee

# $count is how many annotations are chosen.
editor-island-marquee =
    { $count ->
        [one] 1 annotation
       *[other] { $count } annotations
    }
editor-island-marquee-delete = Delete the annotations
editor-island-marquee-confirm-delete = Confirm deleting

## A range drawn across the strip

editor-island-range = Range
# Cuts the range and the time it takes.
editor-island-range-cut = Cut the range
editor-island-range-confirm-cut = Confirm cutting
# Lets go of the range drawn on the timeline, as Command-D or Ctrl-D does.
editor-island-range-deselect = Deselect range
# Opens the Export island set to make a GIF of the range.

## Several beats and lulls gathered on the strip with Shift

# $count is how many beats and lulls are chosen, two or more.
editor-island-strip-group =
    { $count ->
        [one] 1 item
       *[other] { $count } items
    }
# Cuts every chosen beat and lull and the time they take.
editor-island-strip-group-cut = Cut the items
editor-island-strip-group-confirm-cut = Confirm cutting

## A beat on the strip: something that happens in the recording, said as it happens there

# A click on a control. $label is the control's name, such as a button's.
editor-island-beat-click = Click “{ $label }”
# Typing. $label is what was typed.
editor-island-beat-type = Type “{ $label }”
# A click on something the recording does not name.
editor-island-beat-click-unnamed = Click
# Typing whose text the recording does not keep, as typed text never is.
editor-island-beat-type-unnamed = Typing
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
editor-island-moment-no-speech = Nothing was said in this note.
editor-island-moment-transcribe-failed = Could not transcribe this note.
editor-island-moment-waiting = Waiting to transcribe
# The tool a missing transcription model is for, beside the model's size and its download, on the Moment, Captions and Script islands.
editor-island-transcription = Transcription
# While the recording or a voice note is transcribed, on the Captions, Script and Moment islands.
editor-island-transcribing = Transcribing
editor-island-transcribe-failed = Could not transcribe the recording.

## A scene on the strip: how a stretch of the recording is framed

editor-island-scene = Scene
# Names the choice of which group of the scene's options shows beneath its row, for assistive technology.
editor-island-scene-groups = Scene options
# The groups of the scene's options, one shown beneath its row at a time.
editor-island-scene-layout = Layout
editor-island-scene-camera = Camera
editor-island-scene-screen = Screen
# Put one of a pane's figures back: its zoom to the whole picture, its position to the middle.
editor-island-scene-reset-zoom = Reset zoom
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
# Keeps the camera on the speaker's face as they move, for the whole recording or, with the playhead in a scene, for that scene.
editor-island-follow-face = Follow face
# Names the spinner beside Follow face for assistive technology, while the camera looks for the speaker's face.
editor-island-finding-face = Finding your face
editor-island-scene-zoom = Zoom
# Which part of the pane's picture sits in the middle of its box.
editor-island-scene-position = Position
editor-island-scene-position-x = Horizontal position
editor-island-scene-position-y = Vertical position
# The axes the two position fields are for.
editor-island-scene-x = X
editor-island-scene-y = Y
# Something on the picture turned in depth and flat: the screen in a scene, or a placed image.
editor-island-rotation = Rotation
editor-island-rotation-x = X rotation
editor-island-rotation-y = Y rotation
editor-island-rotation-z = Z rotation
editor-island-reset-rotation = Reset rotation
# At the Scene island's foot while no scene is in hand. The scene line runs along the timeline strip.
editor-island-scene-hint = Drag along the scene line to add a scene.
editor-island-scene-auto-zoom = Auto zoom
# Removes every planned zoom; a zoom changed by hand stays.
editor-island-scene-clear-auto-zooms = Clear zooms
editor-island-scene-confirm-clear-auto-zooms = Confirm clearing
# Plans the zooms again, replacing the planned zooms.
editor-island-scene-regenerate-auto-zooms = Regenerate

## An annotation drawn on the picture, in hand

editor-island-annotation-delete = Delete the annotation
editor-island-annotation-confirm-delete = Confirm deleting
# Whether the annotation draws itself in as its bar on the strip begins, rather than appearing whole.
editor-island-annotation-animate = Animate
# Turns on or off whether the annotation follows the content under it as the recording moves, such as a button on a page that scrolls.
editor-island-annotation-pin = Pin to content
# Under a pinned annotation's island while it follows its content at the playhead.
editor-island-annotation-pin-following = Following the content
# Under a pinned annotation's island where its content is out of view at the playhead, so the annotation is hidden there.
editor-island-annotation-pin-out-of-view = Content out of view here
# Beside a progress bar while the path a pinned annotation follows is worked out.
editor-island-annotation-pin-tracking = Tracking the content
# A warning under a pinned annotation's island where the tracker lost the content it follows at the playhead, so where the annotation is there is a guess to check.
editor-island-annotation-pin-lost = Tracking lost the content here
# A warning under a pinned annotation's island where the content it follows is outside the recorded frame at the playhead, so the tracker is unsure of it there.
editor-island-annotation-pin-off-frame = Content off the frame here
# A warning under a pinned annotation's island where the content it follows went under something that stays put on screen, such as a page's sticky header, at the playhead, so the tracker is unsure of it there.
editor-island-annotation-pin-under-cover = Content under a cover here
# Says the pinned annotation's content went out of view at the playhead, hiding the annotation from there.
editor-island-annotation-pin-hide = Out of view
# Says the pinned annotation's content is back in view at the playhead, showing the annotation again from there.
editor-island-annotation-pin-show = Back in view
# Takes away every place the pinned annotation was put by hand, leaving it to follow its content alone.
editor-island-annotation-pin-clear = Clear corrections
# Under an annotation's island while the playhead is in a scene where the annotation was moved: it stands somewhere of its own in this scene, and glides there as the scene arrives and back as it leaves.
editor-island-annotation-scene-placed = Placed for this scene
# Takes away the annotation's own place in the scene at the playhead, so it stands there where it does outside every scene.
editor-island-annotation-scene-reset = Reset place in this scene
editor-island-arrow = Arrow
# Turns the arrow round, so its head points the other way.
editor-island-arrow-reverse = Reverse
editor-island-counter = Counter
# A freehand stroke drawn on the picture.
editor-island-drawing = Drawing
editor-island-focus = Focus
editor-island-highlight = Highlight
# A picture placed on the picture, from a file.
editor-island-image = Image
# Gives the image another picture, keeping its place, size and turn.
editor-island-image-replace = Replace
# Mirrors the image across its upright axis.
editor-island-image-flip = Flip
# A lens that shows the part of the picture under it larger.
editor-island-magnifier = Magnifier
# A box that hides what is under it, by erasing, filling, pixelating or blurring it.
editor-island-redaction = Redaction
# Under a redaction whose settings leave what it covers recoverable or briefly visible: a blur, classic pixelation, or drawing in, which shows what it covers until it arrives.
editor-island-redaction-not-secure = These settings aren't secure.
# Names the row choosing how a redaction hides what is under it: erasing, filling, pixelating or blurring.
editor-island-redaction-style = Style
editor-island-shape = Shape
# A text placed on the picture.
editor-island-text = Text
# The grid of templates a text can take its look from, opened from its template line.
editor-island-text-template = Templates
# The line naming the template in use, which opens the templates; read by assistive technology.
editor-island-text-template-current = Template: { $name }
# A face as its line shows it: its family and its style, such as "Inter Semibold".
editor-island-text-font-value = { $family } { $style }
# The lines naming the faces in use, which open the font lists; read by assistive technology.
editor-island-text-font-current = Font: { $font }
editor-island-text-accent-font-current = Accent font: { $font }
editor-island-text-font-search = Find a font
editor-island-text-font-families = Font families
editor-island-text-font-weight = Weight
# A weight named like another, told apart by its number, such as "Bold 700".
editor-island-text-font-style-detail = { $style } { $weight }
# A toggle: whether a text is set in its family's italic.
editor-island-text-italic = Italic
# The text's words, any of which can be set apart in the template's accent or highlight.
editor-island-text-words = Words set apart
# A toggle: whether a text with no box is outlined.
editor-island-text-outline = Outline
# A toggle: whether a moving text's words come on one after another rather than together.
editor-island-text-word-by-word = Word by word
# Which of a text's colours the grid below sets: its type, its box, its accent and so on.
editor-island-text-color-role = Colour for
# The island for the screen of a recording: how far its edges are extended, its own corners, its shadow and its turn.
editor-island-screen = Screen
# The same island for a screenshot, which is what was captured.
editor-island-screenshot = Screenshot
# The shape a crop keeps as its handles are dragged, on the Crop tool's island.
editor-island-crop-aspect = Aspect ratio
# A crop with no shape kept: its handles drag freely.
editor-island-crop-free = Free
# A crop that keeps the shape a freely drawn crop had, offered after Free on the Crop tool's island once a free crop is drawn.
editor-island-crop-locked = Locked
# A crop that keeps the shape of the whole capture.
editor-island-crop-original = Original
# An aspect ratio as it is written, such as "16:9".
editor-island-crop-ratio = { $width }:{ $height }
# An aspect ratio as it is spoken, such as "16 by 9".
editor-island-crop-ratio-name = { $width } by { $height }
# Gives a layer's whole capture back, free of any crop, on the Crop tool's island.
editor-island-reset-crop = Reset crop
# Downloads the model a tool needs, on any island whose tool needs one.
editor-island-model-download = Download
# Names a model's download progress bar for assistive technology.
editor-island-model-downloading = Downloading model
# How far the screen's edges are extended, filled with the colour found behind its content: not at all, or by a little, more or a lot.
editor-island-screen-inset = Inset
editor-island-screen-inset-none = None
editor-island-screen-inset-tight = Tight
editor-island-screen-inset-medium = Medium
editor-island-screen-inset-large = Large
# Puts what was captured back in the middle of its box, after a crop or an inset moved it off centre.
editor-island-screen-recenter = Recenter
# Puts the screen, a screenshot's layer or the camera back where a new project places it, at the size and in the place it starts at, keeping its crop, inset, corners, shadow and rotation. Over a scene, puts the scene's pane back where the scene's layout puts it.
editor-island-reset-placement = Reset placement
# The island for the camera, for the whole recording: its effects, its corners, its shadow and whether it follows the face.
editor-island-camera = Camera
# The toggle at the end of the Camera island's row, named for assistive technology: on, the camera is drawn into the video; off, it is saved as a file of its own beside it, the Export island's "Save the camera as its own file".
editor-island-camera-in-video = Camera in the video
# The Camera island's note while the camera is saved as a file of its own, so it is not in the picture.
editor-island-camera-separate = Saved as its own file beside the video.
# Shown while picking objects on the preview, where a drag paints a line over an object. $key is the key held to remove instead, drawn as a keycap: Option on a Mac, Alt on Windows.
editor-island-camera-objects-hint = Click or paint to keep. Hold { $key } to remove.
# At the foot of the Screen island while the playhead is in a scene, which can pad, round, turn, recenter and place the screen its own way; also on the Camera island of a recording of the camera alone, whose camera is padded and turned as a screen is.
editor-island-screen-in-scene = Inset, corners, rotation, Recenter and Reset placement apply to this scene only.
# At the foot of the Crop tool's island while the playhead is in a scene, which crops the screen and the camera its own way.
editor-island-crop-in-scene = This crop applies to this scene only.
# At the foot of the Camera island while the playhead is in a scene, which can round the camera, have it follow the face and place it its own way.
editor-island-camera-in-scene = Corners, Follow face and Reset placement apply to this scene only.
# At the foot of the Frame island while the playhead is in a scene, which can sit on a background of its own; the frame's size holds for the whole recording.
editor-island-frame-in-scene = The background applies to this scene only.
# At the foot of the Cursor island while the playhead is in a scene, about the eye on its row, which there shows or hides the cursor for that scene alone; the cursor's other options hold for the whole recording. Kept short, under the island's Cursor title. $state is "hidden" or "shown" where the scene hides or shows the cursor its own way (a game with no cursor, say), and "other" where it does as the rest of the recording does, so the eye would make it the scene's own.
editor-island-cursor-in-scene =
    { $state ->
        [hidden] Hidden in this scene only.
        [shown] Shown in this scene only.
       *[other] Applies to this scene only.
    }
# A dice that makes a frame's background again: a new colour, or new colours for a mesh's unlocked ones, in a mood picked at random.
editor-island-frame-randomise = Randomise
# Puts the frame's size back to the size of what was captured, leaving its background and corners as they are.
editor-island-frame-original-size = Original size
# On the Frame island while the playhead is in a scene that has a background of its own: takes that background away, so the scene shows the one set outside every scene again, and a later change to that one reaches the scene too.
editor-island-frame-clear-scene-background = Clear scene background
# The library's pictures, beside the Frame island, one of which frames the picture.
editor-island-frame-library = Pictures from the library
editor-island-frame-library-empty = Pictures you give images are kept in the library, and can frame the picture from here.

## Shared rows: names, sizes and places several islands use.

editor-island-size = Size
# The two fields of a size, and the toggle between them that keeps their proportions when one is changed.
editor-island-width = Width
editor-island-height = Height
editor-island-keep-aspect-ratio = Keep aspect ratio
editor-island-position = Position
# The nine places on the picture an overlay such as captions or shortcuts can sit, each the accessible name of its cell in the Position grid.
editor-island-position-top-left = Top left
editor-island-position-top = Top
editor-island-position-top-right = Top right
editor-island-position-left = Left
editor-island-position-center = Centre
editor-island-position-right = Right
editor-island-position-bottom-left = Bottom left
editor-island-position-bottom = Bottom
editor-island-position-bottom-right = Bottom right
editor-island-audio = Audio
# A layer of a screenshot, counted from the bottom of the stack.
editor-island-layer = Layer { $number }

## Images placed as annotations

editor-island-replace = Replace

## Cursor

editor-island-cursor = Cursor
editor-island-cursor-size = Cursor size
# A toggle: whether the cursor is cut off at the recording's edge rather than drawn past it. Short, since the Cursor island already says what is kept.
editor-island-cursor-clip = Keep inside
editor-island-cursor-smooth = Smooth movement
editor-island-cursor-motion-blur = Motion blur
editor-island-cursor-click-animation = Click animation
# A toggle drawn as an eye: whether Screenwide draws the cursor over the recording. Inside a scene it shows or hides the cursor in that scene alone.
editor-island-cursor-show = Show cursor

## Keyboard shortcuts drawn over a recording

editor-island-keyboard = Keyboard
editor-island-keyboard-show = Show shortcuts
editor-island-keyboard-light = Light
editor-island-keyboard-restore = Restore all shortcuts

## Transcript: the recording's microphone as words, chosen and cut, or corrected in place

# The tool's tooltip and the transcript's accessible name.
editor-island-transcript = Transcript
# Shown under Transcribe when transcribing the recording failed.
editor-island-transcript-failed = Could not transcribe the recording. Try again.
# The accessible name of the word or words being edited in place.
editor-island-transcript-editing = Word being edited
# The accessible name of the menu a right click on the transcript's words opens.
editor-island-transcript-actions = Word actions
# Menu item: remove the chosen words from the video.
editor-island-transcript-remove = Remove
# Menu item: edit the chosen words in place.
editor-island-transcript-edit = Edit
# Menu item: bring back the struck-through words chosen.
editor-island-transcript-restore = Restore
# Menu item: put the chosen words back to the text that was heard.
editor-island-transcript-restore-spelling = Restore spelling
# The row that removes every filler word: its title.
editor-island-transcript-fillers = Filler words
# The row that removes every word marked as profanity: its title.
editor-island-transcript-profanity = Profanity
# Menu item on a filler or profane word: cut every word written the same way. $text is the word as the transcript shows it.
editor-island-transcript-remove-same = Remove every “{ $text }”
# Menu item on a filler: cut every filler word in the transcript.
editor-island-transcript-remove-fillers = Remove all filler words
# Menu item on a profane word: cut every word marked as profanity.
editor-island-transcript-remove-profanity = Remove all profanity
# Menu item on a word marked as profanity: stop marking it anywhere in this recording.
editor-island-transcript-not-profanity = Not profanity
# Heading over the scene layouts in the words' menu, where the chosen words sit outside every scene: picking one adds a scene over them.
editor-island-transcript-add-scene = Add scene
# Heading over the scene layouts in the words' menu, where the chosen words sit in a scene: picking one changes that scene's layout and fits its timing to the words.
editor-island-transcript-change-scene = Change scene

## Captions: the transcript's words shown a few at a time, timed to the voice

# The accessible name of the transcript as the captions break it, under Show captions.
editor-island-captions-text = Caption text
# The menu a right click on a word of the caption text opens: its accessible name, then each item. A break starts a new caption between two words; a join keeps them in one caption. A key word is the word of its caption set apart in the accent's font, size and colour.
editor-island-captions-actions = Caption actions
editor-island-captions-split = Start a new caption here
editor-island-captions-join = Join with the previous caption
editor-island-captions-clear-split = Clear this break
editor-island-captions-remove-join = Allow a break here
editor-island-captions-hide-word = Hide from captions
editor-island-captions-show-word = Show in captions
editor-island-captions-make-key-word = Make key word
editor-island-captions-not-key-word = Not a key word
editor-island-text-template-box = Box
editor-island-text-template-clean = Clean
editor-island-text-template-pop = Pop
editor-island-text-template-highlight = Highlight
editor-island-text-template-karaoke = Karaoke
editor-island-text-template-punch = Punch
editor-island-text-template-editorial = Editorial
editor-island-text-template-supersize = Supersize
editor-island-text-template-stack = Stack
editor-island-text-template-staircase = Staircase
editor-island-text-template-rise = Rise
editor-island-text-template-stagger = Stagger
editor-island-text-template-shadow = Shadow
editor-island-text-template-offset = Offset
editor-island-text-template-underline = Underline
editor-island-text-template-glow = Glow
editor-island-text-template-flip = Flip
# The few words each template's preview shows, said a couple at a time; short, so they fit a small tile.
editor-island-text-sample = Captions that move with every word
# The app's own font in the font list, ahead of the ones installed on the computer.
editor-island-text-font-built-in = Inter (built in)
editor-island-text-weight-thin = Thin
editor-island-text-weight-extra-light = Extra light
editor-island-text-weight-light = Light
editor-island-text-weight-regular = Regular
editor-island-text-weight-medium = Medium
editor-island-text-weight-semibold = Semibold
editor-island-text-weight-bold = Bold
editor-island-text-weight-extra-bold = Extra bold
editor-island-text-weight-black = Black
# The same weights set in the font's italic, for fonts that have one.
editor-island-text-weight-thin-italic = Thin italic
editor-island-text-weight-extra-light-italic = Extra light italic
editor-island-text-weight-light-italic = Light italic
editor-island-text-weight-regular-italic = Italic
editor-island-text-weight-medium-italic = Medium italic
editor-island-text-weight-semibold-italic = Semibold italic
editor-island-text-weight-bold-italic = Bold italic
editor-island-text-weight-extra-bold-italic = Extra bold italic
editor-island-text-weight-black-italic = Black italic
editor-island-text-color-text = Text
editor-island-text-color-highlight = Highlight
editor-island-text-color-accent = Accent
editor-island-text-color-outline = Outline
editor-island-text-color-background = Box
editor-island-text-color-shadow = Shadow

## Scenes: how the screen and camera are laid out over time

# A camera switch that cuts the person out of the room behind them, by a downloaded model.
editor-island-remove-background = Remove background
editor-island-remove-background-working = Removing background
# What is behind the person once the room is removed: nothing, the room blurred, or a picture.
editor-island-camera-background = Background
editor-island-camera-background-none = None
editor-island-camera-background-blur = Blur
editor-island-camera-background-picture = Picture
editor-island-replace-camera-picture = Replace the background picture
# Objects on camera, picked to keep with the person or to remove with the room.
editor-island-camera-objects = Objects
editor-island-camera-objects-pick = Pick
editor-island-camera-objects-done = Done
# Laying the objects followed into the cutout of the person.
editor-island-camera-objects-working = Applying objects
# A dot standing for one object picked; pressing it goes to where it was picked.
editor-island-camera-object-kept = Kept object
editor-island-camera-object-removed = Removed object
# The ring round a dot while its object is followed through the recording.
editor-island-camera-object-following = Following through the recording
# The bin beside the dots, for the object chosen among them.
editor-island-camera-object-remove = Remove the chosen object
editor-island-camera-object-confirm-remove = Confirm removing the object
# A camera switch that sharpens the camera, enlarges it along its edges where it is drawn larger than it was recorded, and corrects its white balance and exposure with a natural look.
editor-island-enhance = Enhance
# Shown on the Enhance switch while the recording is measured for its colour.
editor-island-enhance-working = Measuring colour

## Audio

editor-island-studio-sound = Studio sound
editor-island-studio-sound-working = Rebuilding voice
editor-island-reduce-noise = Reduce noise
editor-island-noise-cleaning = Removing noise
editor-island-vocal-cleanup = Vocal cleanup
editor-island-voice-cleaning = Cleaning up voice
editor-island-auto-volume = Auto volume
editor-island-auto-volume-measuring = Measuring volume
# A microphone switch that lowers background audio, singers and speakers most, so the person talking stays clear.
editor-island-studio-ducking = Studio ducking
editor-island-studio-ducking-working = Finding vocals
