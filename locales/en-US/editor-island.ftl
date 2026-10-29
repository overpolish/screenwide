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

## The recording's script, what was said as a whole

# The island for the recording's script, open beside the Captions island.
editor-island-script = Script
# Under the script while it is open with the captions: how to edit it.
editor-island-script-captions-hint = Double-click a word to edit it. Right-click for breaks, hiding and key words.
# Takes back every edit made in the script: breaks, hidden or shown words and key words placed by hand, and spelling fixes, which go back to what was heard.
editor-island-script-reset = Reset caption edits

## The recording's captions, all of them at once

# The island for the recording's captions: their look, their layout and what they say.
editor-island-captions = Captions
# A toggle in the island's row: whether the captions are drawn over the recording.
editor-island-captions-show = Show captions
# Names the type size choice for assistive technology: a step from the template's own size.
editor-island-captions-size = Size
# Names the key word's size choice, shown where the template sets one word apart.
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
editor-island-captions-transcribing = Transcribing
editor-island-captions-transcribe-failed = Could not transcribe the recording.
editor-island-captions-no-speech = No words detected.

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
# The tool a missing transcription model is for, beside the model's size and its download, on the Moment and Captions islands.
editor-island-transcription = Transcription

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
# A loupe that shows part of the picture larger.
editor-island-magnifier = Magnifier
# A box that hides what is under it, by erasing, filling, pixelating or blurring it.
editor-island-redaction = Redaction
# Under a redaction whose settings leave what it covers recoverable or briefly visible: a blur, classic pixelation, or drawing in, which shows what it covers until it arrives.
editor-island-redaction-not-secure = These settings aren't secure.
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
# The island for the screen of a recording: the shape it is cropped to, how far its edges are extended, its own corners and its shadow.
editor-island-screen = Screen
# The same island for a screenshot, which is what was captured.
editor-island-screenshot = Screenshot
# The shape a crop keeps as its handles are dragged, on the Screen and Camera islands.
editor-island-crop-aspect = Aspect ratio
# A crop with no shape kept: its handles drag freely.
editor-island-crop-free = Free
# A crop that keeps the shape of the whole capture.
editor-island-crop-original = Original
# An aspect ratio as it is written, such as "16:9".
editor-island-crop-ratio = { $width }:{ $height }
# An aspect ratio as it is spoken, such as "16 by 9".
editor-island-crop-ratio-name = { $width } by { $height }
# Gives a layer's whole capture back, free of any crop, on the Screen and Camera islands.
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
# The island for the camera, for the whole recording: its crop, its effects, its corners, its shadow and whether it follows the face.
editor-island-camera = Camera
# At the foot of the Screen island while the playhead is in a scene, which can round the screen its own way.
editor-island-corners-in-scene = Corners apply to this scene only.
# At the foot of the Camera island while the playhead is in a scene, which can round the camera and have it follow the face its own way.
editor-island-camera-in-scene = Corners and Follow face apply to this scene only.
# A dice that makes a frame's background again: a new colour, or new colours for a mesh's unlocked ones, in a mood picked at random.
editor-island-frame-randomise = Randomise
# Puts the frame's size back to the size of what was captured, leaving its background and corners as they are.
editor-island-frame-original-size = Original size
