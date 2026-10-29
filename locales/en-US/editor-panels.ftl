# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor's tool panels: the controls each tool shows while it is in
### hand. Row titles are short; most are one or two words.

editor-panels-nothing-selected = Nothing selected
editor-panels-size = Size
# The two fields of a size, and the toggle between them that keeps their proportions when one is changed.
editor-panels-width = Width
editor-panels-height = Height
editor-panels-keep-aspect-ratio = Keep aspect ratio
editor-panels-position = Position
# The nine places on the picture an overlay such as captions or shortcuts can sit, each the accessible name of its cell in the Position grid.
editor-panels-position-top-left = Top left
editor-panels-position-top = Top
editor-panels-position-top-right = Top right
editor-panels-position-left = Left
editor-panels-position-center = Centre
editor-panels-position-right = Right
editor-panels-position-bottom-left = Bottom left
editor-panels-position-bottom = Bottom
editor-panels-position-bottom-right = Bottom right
editor-panels-inset = Inset
editor-panels-drop-shadow = Drop shadow
editor-panels-reset = Reset
editor-panels-recenter = Recenter
editor-panels-delete = Delete
editor-panels-clear-all = Clear all
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
editor-panels-keyboard-animation = Animation
editor-panels-keyboard-pop = Pop
editor-panels-keyboard-fade = Fade
editor-panels-keyboard-none = None
editor-panels-keyboard-appearance = Appearance
editor-panels-keyboard-dark = Dark
editor-panels-keyboard-light = Light
editor-panels-keyboard-restore = Restore all shortcuts

## Transcript: the recording's microphone as words, chosen and cut, or corrected in place

# The tool's tooltip and the transcript's accessible name.
editor-panels-transcript = Transcript
# The toolbar button's accessible name.
editor-panels-transcript-name = Transcript editing
# Shown beside the download of the model the tool needs.
editor-panels-transcript-description = Text-driven editing.
editor-panels-transcript-transcribe = Transcribe
editor-panels-transcript-cancel = Cancel
editor-panels-transcript-working = Transcribing
# Shown under Transcribe when transcribing the recording failed.
editor-panels-transcript-failed = Could not transcribe the recording. Try again.
# $key is drawn as a keycap: Command on macOS, Ctrl on Windows.
editor-panels-transcript-hint = { $key } click/drag to remove or restore words. Double-click to edit.
# The accessible name of the word or words being edited in place.
editor-panels-transcript-editing = Word being edited
# The accessible name of the menu a right click on the transcript's words opens.
editor-panels-transcript-actions = Word actions
# Menu item: remove the chosen words from the video.
editor-panels-transcript-remove = Remove
# Menu item: edit the chosen words in place.
editor-panels-transcript-edit = Edit
# Menu item: bring back the struck-through words chosen.
editor-panels-transcript-restore = Restore
# Menu item: put the chosen words back to the text that was heard.
editor-panels-transcript-restore-spelling = Restore spelling
# The search field over the transcript's words: its placeholder and accessible name.
editor-panels-transcript-search = Search transcript
# The row that removes every filler word: its title.
editor-panels-transcript-fillers = Filler words
# The row that removes every word marked as profanity: its title.
editor-panels-transcript-profanity = Profanity
# Under the filler words and profanity rows, while the transcript has any: how many.
editor-panels-transcript-found =
    { $count ->
        [one] { $count } found
       *[other] { $count } found
    }
# Menu item on a filler or profane word: cut every word written the same way. $text is the word as the transcript shows it.
editor-panels-transcript-remove-same = Remove every “{ $text }”
# Menu item on a filler: cut every filler word in the transcript.
editor-panels-transcript-remove-fillers = Remove all filler words
# Menu item on a profane word: cut every word marked as profanity.
editor-panels-transcript-remove-profanity = Remove all profanity
# Menu item on a word marked as profanity: stop marking it anywhere in this recording.
editor-panels-transcript-not-profanity = Not profanity
# Heading over the scene layouts in the words' menu, where the chosen words sit outside every scene: picking one adds a scene over them.
editor-panels-transcript-add-scene = Add scene
# Heading over the scene layouts in the words' menu, where the chosen words sit in a scene: picking one changes that scene's layout and fits its timing to the words.
editor-panels-transcript-change-scene = Change scene
# The chevron that expands the Replace all row under the search field.
editor-panels-transcript-replace-toggle = Replace
# The text field for what every match is replaced with.
editor-panels-transcript-replace-with = Replace with
# Button that replaces every match found, as one undo step.
editor-panels-transcript-replace-all = Replace all
# Button that transcribes the recording again, once it already has a transcript.
editor-panels-transcript-transcribe-again = Transcribe again

## Captions: the transcript's words shown a few at a time, timed to the voice

# The tool's tooltip.
editor-panels-captions = Captions
# The toolbar button's accessible name.
editor-panels-captions-name = Captions from the transcript
# Shown beside the transcript's model download and Transcribe, while there is no transcript to caption.
editor-panels-captions-description = Captions are made from the transcript.
# The accessible name of the spinner the Captions panel shows while it comes up.
editor-panels-captions-loading = Loading captions
editor-panels-captions-show = Show captions
# Brings back every caption deleted from the timeline or the picture. Deleting a caption only takes it out of the video and the subtitles; the transcript keeps its words.
editor-panels-captions-restore = Restore all captions
# The accessible name of the transcript as the captions break it, under Show captions.
editor-panels-captions-text = Caption text
# Under the caption text: take away every break, join, hidden or shown word and key word placed by hand. Spelling fixes stay.
editor-panels-captions-reset-text = Reset breaks, hidden words and key words
# The menu a right click on a word of the caption text opens: its accessible name, then each item. A break starts a new caption between two words; a join keeps them in one caption. A key word is the word of its caption set apart in the accent's font, size and colour.
editor-panels-captions-actions = Caption actions
editor-panels-captions-split = Start a new caption here
editor-panels-captions-join = Join with the previous caption
editor-panels-captions-clear-split = Clear this break
editor-panels-captions-remove-join = Allow a break here
editor-panels-captions-hide-word = Hide from captions
editor-panels-captions-show-word = Show in captions
editor-panels-captions-make-key-word = Make key word
editor-panels-captions-not-key-word = Not a key word
# The Captions panel's three tabs: the group's accessible name, then each tab. Words holds what captions say and how long they stay up; Style holds the template and how it looks; Layout holds the type's size and case, and how many words and lines a caption holds.
editor-panels-captions-tabs = Caption settings
editor-panels-captions-tab-words = Words
editor-panels-captions-tab-style = Style
editor-panels-captions-tab-layout = Layout
# The caption templates: the group's accessible name, and each one's name under its moving preview.
editor-panels-text-template = Caption style
# The Style tab's row whose button opens the caption templates to choose from.
editor-panels-text-template-picker = Template
editor-panels-text-template-box = Box
editor-panels-text-template-clean = Clean
editor-panels-text-template-pop = Pop
editor-panels-text-template-highlight = Highlight
editor-panels-text-template-karaoke = Karaoke
editor-panels-text-template-punch = Punch
editor-panels-text-template-editorial = Editorial
editor-panels-text-template-supersize = Supersize
editor-panels-text-template-stack = Stack
editor-panels-text-template-staircase = Staircase
editor-panels-text-template-rise = Rise
editor-panels-text-template-stagger = Stagger
editor-panels-text-template-shadow = Shadow
editor-panels-text-template-offset = Offset
editor-panels-text-template-underline = Underline
editor-panels-text-template-glow = Glow
editor-panels-text-template-flip = Flip
# The few words each template's preview shows, said a couple at a time; short, so they fit a small tile.
editor-panels-text-sample = Captions that move with every word
# The face caption type is set in, chosen from a list of the fonts installed on the computer and those the app carries.
editor-panels-text-font = Font
# The placeholder and accessible name of the field heading the font list, which finds a font by name.
editor-panels-text-font-search = Find a font
# The app's own font in the font list, ahead of the ones installed on the computer.
editor-panels-text-font-built-in = Inter (built in)
# How heavy the caption type is, from the weights its font has, lightest first, by the names type designers give them.
editor-panels-text-weight = Weight
editor-panels-text-weight-thin = Thin
editor-panels-text-weight-extra-light = Extra light
editor-panels-text-weight-light = Light
editor-panels-text-weight-regular = Regular
editor-panels-text-weight-medium = Medium
editor-panels-text-weight-semibold = Semibold
editor-panels-text-weight-bold = Bold
editor-panels-text-weight-extra-bold = Extra bold
editor-panels-text-weight-black = Black
# The same weights set in the font's italic, for fonts that have one.
editor-panels-text-weight-thin-italic = Thin italic
editor-panels-text-weight-extra-light-italic = Extra light italic
editor-panels-text-weight-light-italic = Light italic
editor-panels-text-weight-regular-italic = Italic
editor-panels-text-weight-medium-italic = Medium italic
editor-panels-text-weight-semibold-italic = Semibold italic
editor-panels-text-weight-bold-italic = Bold italic
editor-panels-text-weight-extra-bold-italic = Extra bold italic
editor-panels-text-weight-black-italic = Black italic
# The font a template's key word is set in, for templates that set one word of each caption apart, its weight's accessible name, and how large it is as a share of the caption's type.
editor-panels-text-accent = Accent
editor-panels-text-accent-weight = Accent weight
editor-panels-captions-accent-size = Accent size
# The steps of a text size slider, as a screen reader names them; Medium is the template's own size.
editor-panels-text-size-small = Small
editor-panels-text-size-medium = Medium
editor-panels-text-size-large = Large
editor-panels-text-size-extra-large = Extra large
# Which colour of the caption the colour grid changes, and each choice: the type, the word being said, the accent, the outline around the type, and the box behind the lines.
editor-panels-text-color = Colour
editor-panels-text-color-text = Text
editor-panels-text-color-highlight = Highlight
editor-panels-text-color-accent = Accent
editor-panels-text-color-outline = Outline
editor-panels-text-color-background = Box
editor-panels-text-color-shadow = Shadow
# A text annotation's row choosing which of its words the template sets apart: in its accent, for templates that set a key word apart, or highlighted, for templates that highlight words.
editor-panels-text-key-words = Key words
editor-panels-text-highlighted-words = Highlighted words
editor-panels-text-chosen-words-description = Choose the words the template sets apart.
# The Style tab's row turning which way the words' shadow falls, where the template casts one.
editor-panels-text-shadow = Shadow
# In each Captions tab: start new captions as these are now. On the Style tab, this template with this look; on the Words tab, these rules.
editor-panels-captions-set-default = Set as default
# In a caption template's menu: put the template back to how it ships, dropping any look set as its default.
editor-panels-text-reset-original = Reset to original
# Also in a caption template's menu: bring in a template someone shared, as a zip or its folder; save this one as a zip to share; and, for one you brought in, move it to the Trash.
editor-panels-text-template-import = Import template
editor-panels-text-template-export = Export template
editor-panels-text-template-remove = Remove template
# Mark the word being said in a caption: in the highlight colour, or on a pill where the template sets it on one.
editor-panels-text-highlight-word = Highlight spoken word
# Show each word of a caption as it is said, so the caption builds up with the voice.
editor-panels-text-reveal-words = Reveal word by word
# A soft outline around the caption's type, so it reads over any picture.
editor-panels-text-outline = Outline
# The Style tab's row of options a caption's look turns on or off, and the short names of the first two as its segments show them.
editor-panels-text-effects = Effects
editor-panels-text-effect-highlight = Highlight
editor-panels-text-effect-word-by-word = Word by word
# The most words one caption shows at once, across its lines.
editor-panels-captions-max-words = Max words
editor-panels-captions-lines = Lines
editor-panels-captions-one-line = One
editor-panels-captions-two-lines = Two
# Keep each caption on screen through a silence until the next one starts.
editor-panels-captions-hold-until-next = Hold until next caption
editor-panels-captions-all-caps = All caps
editor-panels-captions-hide-punctuation = Hide punctuation
editor-panels-captions-hide-fillers = Hide filler words
# How a word marked as profanity shows: the row's title, then each choice.
editor-panels-captions-profanity-style = Profanity
# Profanity shown as it was said.
editor-panels-captions-profanity-said = As said
# Profanity shown with its first letter and the rest starred out.
editor-panels-captions-profanity-partly = Partly
# Profanity shown with every letter starred out.
editor-panels-captions-profanity-fully = Fully
# Profanity left out of the caption.
editor-panels-captions-profanity-hidden = Hidden

## Scenes: how the screen and camera are laid out over time

editor-panels-scene = Scene
editor-panels-scene-needs-screen = Scenes need a recording with a screen.
editor-panels-scene-paused = This scene is paused while the camera is left out.
editor-panels-output = Output
editor-panels-output-combined = One video
editor-panels-output-separate = Separate files
editor-panels-swap = Swap
# A switch that keeps the camera's crop on the person's face as they move.
editor-panels-follow-face = Follow face
editor-panels-follow-face-finding = Finding your face in the camera.
editor-panels-zoom = Zoom
# $pane is the part of the scene being framed: Screen or Camera.
editor-panels-pane-zoom = { $pane } zoom
editor-panels-pane-radius = { $pane } radius
editor-panels-pane-x = { $pane } X position
editor-panels-pane-y = { $pane } Y position
editor-panels-rotation = Rotation
editor-panels-rotation-x = X rotation
editor-panels-rotation-y = Y rotation
editor-panels-rotation-z = Z rotation
editor-panels-reset-rotation = Reset rotation
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
# A camera switch that cuts the person out of the room behind them, by a downloaded model.
editor-panels-remove-background = Remove background
editor-panels-remove-background-working = Removing background
editor-panels-remove-background-description = Removes the background behind you.
# What is behind the person once the room is removed: nothing, the room blurred, or a picture.
editor-panels-camera-background = Background
editor-panels-camera-background-none = None
editor-panels-camera-background-blur = Blur
editor-panels-camera-background-picture = Picture
# The picture shown behind the person on camera.
editor-panels-camera-picture = Picture
editor-panels-replace-camera-picture = Replace the background picture
# Objects on camera, picked to keep with the person or to remove with the room.
editor-panels-camera-objects = Objects
editor-panels-camera-objects-description = Keep or remove objects on camera.
editor-panels-camera-objects-pick = Pick
editor-panels-camera-objects-done = Done
# Laying the objects followed into the cutout of the person.
editor-panels-camera-objects-working = Applying objects
# Shown while picking objects on the preview, where a drag paints a line over an object. $platform is windows or macos: the remove modifier is Alt or Option.
editor-panels-camera-objects-hint =
    { $platform ->
        [windows] Click or paint to keep. Hold Alt to remove.
       *[other] Click or paint to keep. Hold Option to remove.
    }
# A dot standing for one object picked; pressing it goes to where it was picked.
editor-panels-camera-object-kept = Kept object
editor-panels-camera-object-removed = Removed object
# The ring round a dot while its object is followed through the recording.
editor-panels-camera-object-following = Following through the recording
# The bin beside the dots, for the object chosen among them.
editor-panels-camera-object-remove = Remove the chosen object
editor-panels-camera-object-confirm-remove = Confirm removing the object
# A camera switch that sharpens the camera, enlarges it along its edges where it is drawn larger than it was recorded, and corrects its white balance and exposure with a natural look.
editor-panels-enhance = Enhance
# Shown on the Enhance switch while the recording is measured for its colour.
editor-panels-enhance-working = Measuring colour

## Audio

editor-panels-volume = Volume
editor-panels-reset-volume = Reset volume
editor-panels-studio-sound = Studio sound
editor-panels-studio-sound-working = Rebuilding voice
editor-panels-studio-sound-description = Enhances your voice.
editor-panels-model-download = Download
editor-panels-model-downloading = Downloading model
editor-panels-reduce-noise = Reduce noise
editor-panels-noise-cleaning = Removing noise
editor-panels-vocal-cleanup = Vocal cleanup
editor-panels-voice-cleaning = Cleaning up voice
editor-panels-auto-volume = Auto volume
editor-panels-auto-volume-measuring = Measuring volume
# A microphone switch that lowers background audio, singers and speakers most, so the person talking stays clear.
editor-panels-studio-ducking = Studio ducking
editor-panels-studio-ducking-working = Finding vocals
editor-panels-studio-ducking-description = Keeps your voice clear over other audio.
editor-panels-silences = Silences
editor-panels-remove = Remove
editor-panels-restore-all = Restore all
editor-panels-restore-silences = Restore all silences
editor-panels-silences-finding = Finding silences
editor-panels-silences-none = No long silences found
# $duration is how much shorter the cuts make the recording, such as 00:48.
editor-panels-silences-removed =
    { $count ->
        [one] { $count } cut, { $duration } shorter
       *[other] { $count } cut, { $duration } shorter
    }
