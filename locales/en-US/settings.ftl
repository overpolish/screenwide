# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The Settings window. Descriptions under a setting stay short: ten words
### or fewer where possible.

settings-sections = Settings sections
settings-add-moment = Add Moment
# Under a feature's group in the Shortcuts pane while the feature is turned off; its shortcuts are greyed out. $feature is the feature's name, such as Ruler.
settings-shortcuts-feature-off = Turn on { $feature } to use these shortcuts.
settings-recording = Recording
settings-screenshots = Screenshots
# The Shortcuts pane's group for Keep Awake, the tray's switch that stops the computer sleeping when idle.
settings-keep-awake = Keep Awake
# The Shortcuts pane's group for dictation, which types what is said into any app.
settings-dictation = Dictation
# The Shortcuts pane's group for notes of a call.
settings-notes = Notes

## Sidebar sections, also the window's title

settings-section-general = General
settings-section-tools = Tools
settings-section-glide = Glide
settings-section-ruler = Ruler
settings-section-annotate = Annotate
settings-section-ocr = OCR
settings-section-moments = Moments
settings-section-transcription = Models
settings-section-shortcuts = Shortcuts
settings-section-licences = Licences

## General

# The group at the end of General holding the app's version and its updates.
settings-about = About
settings-app-name = { -app-name }
settings-version = Version { $version }
settings-update-to = Update to v{ $version }
# The button under Software Update when the Microsoft Store has a new version but did not say which.
settings-update = Update
settings-check-updates = Check for Updates
settings-saving = Saving
settings-default-folder = Default folder
settings-projects-folder = Projects folder
settings-projects-folder-description = New recordings and their edits are kept here.
settings-projects-folder-reset = Use the default project folder
settings-recording-folder = Recording folder
settings-recording-folder-reset = Use the default recording folder
settings-screenshot-folder = Screenshot folder
settings-screenshot-folder-reset = Use the default screenshot folder
settings-export-folder-description = You can choose a different folder when saving.
settings-capture = Capture
settings-include-app = Include { -app-name } in captures
settings-include-app-description = Show { -app-name }'s own windows in recordings and screenshots.
settings-hide-desktop-icons = Hide desktop icons
# On macOS the icons and widgets are left out of the capture and stay on screen; on Windows the icons are hidden on screen while recording.
settings-hide-desktop-icons-description = { $platform ->
    [windows] Hidden from the screen while you record it.
   *[other] Left out of recordings and screenshots, with widgets.
}
settings-hidden-apps = Hidden apps
# On Windows the apps' windows are hidden from every capture, screen sharing included, while a recording runs.
settings-hidden-apps-description = { $platform ->
    [windows] Hidden from recordings and screen sharing while recording.
   *[other] Left out of recordings and screenshots.
}
# Shown in place of the apps' icons when none is running that can be hidden.
settings-hidden-apps-empty = No apps are running that can be hidden.
# Names a hidden app's icon when the app is not running now.
settings-hidden-apps-not-running = { $app }, not running
settings-screenshot-delay = Delayed Screenshot
settings-startup = Startup
settings-launch-at-login = Start when you sign in
settings-show-controls-on-launch = Show recording controls on startup

## Recording

settings-section-recording = Recording
# The group of settings for the island while a recording runs.
settings-recording-controls = Recording controls
# The group of settings for how the recording itself is made.
settings-recording-video = Video
settings-confidence-checks = Show camera and sound while recording
settings-confidence-checks-description = Check your camera, microphone and computer sound are working.
settings-recording-size = Show file size while recording
settings-recording-size-description = See how much disk space the recording takes up.
settings-auto-zoom = Zoom in automatically
settings-auto-zoom-description = Adds zooms for you, always editable in the editor.
settings-countdown = Recording countdown
settings-countdown-off = Off
settings-frame-rate = Frame rate
settings-frame-rate-description = More frames look smoother and make a larger file.
settings-fps = { $fps } fps
settings-anti-flicker = Camera anti-flicker
settings-anti-flicker-description = Stops camera flicker under 50 Hz lights.
# $platform is macos or windows: the bar along the top on macOS, the taskbar on Windows.
settings-hide-menu-bar = { $platform ->
    [windows] Hide taskbar
   *[other] Hide menu bar
}
settings-hide-menu-bar-description = { $platform ->
    [windows] Crops it out of screen recordings, unless set to hide.
   *[other] Crops it and its recording icons out of screen recordings.
}
settings-hide-dock = Hide Dock
settings-hide-dock-description = Crops it out of screen recordings, unless set to hide.

## Glide

settings-glide-enabled = Move and resize windows to fit your screen
settings-glide-mouse-modifier = Glide control for mouse
settings-glide-mouse-modifier-description = Hold while moving from a window's top bar.
settings-glide-thirds-modifier = Control for screen thirds
settings-glide-thirds-modifier-description = Hold during Glide to use thirds instead of halves.
# $platform is macos or windows: macOS calls virtual desktops Spaces.
settings-glide-spaces-modifier =
    { $platform ->
        [macos] Modifier for Spaces
       *[other] Modifier for virtual desktops
    }
settings-glide-spaces-modifier-description =
    { $platform ->
        [macos] Hold while moving between Spaces.
       *[other] Hold while moving between virtual desktops.
    }
settings-glide-monitors-modifier = Modifier for Monitors
settings-glide-monitors-modifier-description = Hold while moving between monitors.
settings-glide-window-gap = Space around windows
settings-glide-cursor-follows = Move pointer with window
settings-glide-double-tap = Double-tap or double-click to center
settings-glide-double-tap-description = Two fingers, or hold mouse control while double-clicking.

## Ruler

settings-ruler-enabled = Measure sizes and distances on your screen
settings-ruler-show = Show ruler
settings-ruler-crosshair = Toggle crosshair
settings-ruler-copy-colour = Copy colour
settings-ruler-copy-colour-description = Copies the colour under the pointer as a hex code.
settings-ruler-delete = Delete measurement
settings-ruler-delete-description = Hover a label or line; otherwise deletes the latest measurement.
settings-ruler-copy = Copy measurement
settings-ruler-copy-description = Copies the latest measurement, regardless of what you hover.
settings-ruler-undo = Undo
settings-ruler-redo = Redo
settings-ruler-stamp-horizontal = Horizontal stamp
settings-ruler-stamp-vertical = Vertical stamp
settings-ruler-stamp-description = Press to stamp; or hold, move, and release.
settings-ruler-guide-vertical = Vertical guide
settings-ruler-guide-vertical-description = Hold and click to place vertical guides.
settings-ruler-guide-horizontal = Horizontal guide
settings-ruler-guide-horizontal-description = Hold and click to place horizontal guides.
settings-ruler-tolerance = Cycle tolerance
settings-ruler-tolerance-description = Switch between clear, balanced, and subtle edge detection.
settings-ruler-radius = Measure radius
settings-ruler-radius-description = Hold over a corner to measure its radius.
settings-ruler-centerlines = Toggle centerlines

## Annotate

settings-annotate-enabled = Draw on your screen
settings-annotate-enabled-description = The annotations are kept in recordings.
settings-annotate-draw = Draw on the screen
settings-annotate-keep = Keep annotations after exiting
settings-annotate-keep-description = They stay on screen until you clear them.
settings-annotate-clear = Clear annotations

## OCR: reading text and QR codes on screen

settings-ocr-enabled = Select text or a QR code on your screen
settings-ocr-activate = Read text or a QR code
settings-ocr-select-all = Select all text
settings-ocr-select-all-description = Select all recognized text.
settings-ocr-copy = Copy text
settings-ocr-copy-description = Copy selected text, or everything when nothing is selected.

## Moments: points flagged while recording, each kind with a colour and a
## shortcut. $moment is a kind's name, which the user chooses.

settings-new-moment = New Moment
settings-moment-name = { $moment } name
# The button on a moment's line showing its shortcut, which opens it in the Shortcuts pane. $keys are its keys joined with " + ", or Not set.
settings-moment-shortcut-open = { $moment } shortcut: { $keys }. Change it in Shortcuts.
settings-moment-color = { $moment } colour
settings-moment-remove = Remove { $moment }
settings-moment-confirm-remove = Confirm removing { $moment }
settings-moments-footnote = Press a moment's shortcut while recording to place it.
settings-voice-notes = Voice Notes
settings-voice-notes-enabled = Voice notes
settings-voice-notes-enabled-description = Hold a moment's shortcut to record a note.
settings-voice-notes-microphone = Microphone
settings-voice-notes-microphone-description = Same as Recording falls back to the system default.
settings-voice-notes-microphone-level = Microphone level
settings-voice-notes-same-microphone = Same as Recording
settings-voice-notes-transcription = Transcription
# $model is a transcription model's name; $size its download size.
settings-voice-notes-needs-model = Needs the { $model } model ({ $size }).
settings-voice-notes-microphone-denied = { -app-name } needs microphone access for voice notes.

## Models: transcription, Restore voice, Smart ducking and background removal. $model is a model's name.

settings-transcription-local = Models run on this computer. Recordings are never uploaded.
# The model the microphone's Restore voice switch uses, named for what it does.
settings-transcription-restore-voice = Restore voice
settings-transcription-restore-voice-description = Makes your voice sound like a studio recording.
# The model the microphone's Smart ducking switch uses, named for what it does.
settings-transcription-smart-ducking = Smart ducking
settings-transcription-smart-ducking-description = Keeps your voice clear over other audio.
# The model the camera's Remove background switch uses, named for what it does.
settings-transcription-camera-background = Background removal
settings-transcription-camera-background-description = Removes the background behind you on camera.
settings-transcription-camera-objects = Camera objects
settings-transcription-camera-objects-description = Keeps or removes objects on camera.
# The model the recording's transcript and voice notes are made with, named for what it does.
settings-transcription-transcripts = Transcripts
settings-transcription-transcripts-description = Turns your recordings and voice notes into text.
# The model dictation takes in the transcription language, named for what it does. The Teleprompter follows the reader's voice with it too.
settings-transcription-dictation = Dictation
# In English, where the words appear as they are spoken.
settings-transcription-dictation-description = Types what you say as you speak, and lets the Teleprompter follow you.
# In a language whose text is written each time the speaker pauses.
settings-transcription-dictation-at-pauses-description = Types what you say at each pause, and lets the Teleprompter follow you.
# In a language Dictation writes with the Transcripts model, downloaded once for both.
settings-transcription-dictation-shared-description = Types what you say at each pause, and lets the Teleprompter follow you. Uses the Transcripts model.
# Shown in place of the description while macOS has not yet given dictation the access it needs.
settings-transcription-dictation-access = Allow microphone and Accessibility access to dictate.
settings-transcription-dictation-allow-microphone = Allow Microphone
settings-transcription-dictation-allow-accessibility = Allow Accessibility
# The model that applies corrections the speaker makes while dictating in English, such as "at 2, actually 3".
settings-transcription-dictation-cleanup = Dictation corrections
settings-transcription-dictation-cleanup-description = Applies corrections you make as you speak, such as “2, actually 3”.
settings-transcription-notes = Meeting notes
settings-transcription-notes-description = Writes notes of your calls and tells the speakers apart.
settings-transcription-language = Language
settings-transcription-language-description = The language you record and dictate in.
# The group of Tools' Dictation section listing names and terms speech is written with.
settings-words = Words
settings-words-description = Names and terms that dictation and transcripts spell as you do here.
# Under the list. $phrase is the quoted phrase that adds a word, in the language dictation is in.
settings-words-by-voice = While dictating, say { $phrase } and spell a word to add it.
settings-words-new = Add a word
settings-words-add = Add
# Names a word's field for assistive technology.
settings-words-edit = Edit { $word }
settings-words-remove = Remove { $word }
# $language is the computer's own language.
settings-transcription-system-language = System ({ $language })
settings-transcription-detect = Detect Automatically
settings-transcription-download = Download
settings-transcription-cancel = Cancel
settings-transcription-downloading = Downloading the { $model } model
# Fetches a fresh copy of a downloaded model, for one that misbehaves.
settings-transcription-download-again = Download the { $model } model again
settings-transcription-remove = Remove the { $model } model
settings-transcription-confirm-remove = Confirm removing the { $model } model

## Keyboard shortcuts

settings-shortcut-toggle-controls = Show or hide recording controls
settings-shortcut-start-stop = Start or stop recording
settings-shortcut-pause-resume = Pause or resume recording
settings-shortcut-mute-microphone = Mute or unmute the microphone
settings-shortcut-mute-microphone-description = Works while recording or taking notes.
settings-shortcut-save-replay = Save replay
settings-shortcut-save-replay-description = Works while the replay buffer is on.
settings-shortcut-screenshot = Take a screenshot
settings-shortcut-screenshot-description = Copies an area, ready to paste, save or edit.
settings-shortcut-reference-screenshot = Keep a reference
settings-shortcut-reference-screenshot-description = Choose an area to keep on screen at its true size.
settings-shortcut-keep-awake = Turn Keep Awake on or off
settings-shortcut-keep-awake-description = Keeps the computer awake. The display still turns off.
settings-shortcut-keep-screen-on = Keep the screen on
settings-shortcut-keep-screen-on-description = Keeps the computer awake and the display on while unlocked.
settings-shortcut-dictate = Dictate
settings-shortcut-dictate-description = Hold to talk, or tap to start and stop.
settings-shortcut-take-notes = Take notes
settings-shortcut-take-notes-description = Starts notes of a call, or stops them.
# The row of a moment's shortcut in the Shortcuts pane. $moment is the moment's name, which the user chooses.
settings-shortcut-place-moment = Place { $moment }

## The editor's and Annotate's own keys, built in: the Shortcuts pane shows
## them in scopes of their own but does not set them.

# The groups the editor's keys are shown in.
settings-shortcuts-editor-tools = Tools
settings-shortcuts-editor-modes = Modes
settings-shortcuts-editor-editing = Editing
settings-shortcuts-editor-timeline = Timeline
# The mode for the recording's screen, or in a screenshot for the screenshot itself.
settings-shortcut-editor-screen = Screen or screenshot
settings-shortcut-editor-undo = Undo
settings-shortcut-editor-redo = Redo
# Copies the chosen annotations, or with none the screenshot or the frame.
settings-shortcut-editor-copy = Copy
settings-shortcut-editor-cut = Cut the chosen annotations
settings-shortcut-editor-paste = Paste annotations or an image
settings-shortcut-editor-select-all = Choose every annotation
settings-shortcut-editor-delete = Delete what is chosen
# Escape puts the tool in hand down and lets go of what is chosen.
settings-shortcut-editor-back-to-select = Back to Select
settings-shortcut-editor-duplicate = Duplicate the chosen annotations, or let go of everything
# Adds the frame under the playhead, or the screenshot, to the open teleprompter's script, as a picture shown beside the word being read.
settings-shortcut-editor-add-cue = Add to the script as a cue
settings-shortcut-editor-confirm-crop = Finish cropping
settings-shortcut-editor-nudge = Nudge what is chosen
settings-shortcut-editor-nudge-further = Nudge what is chosen further
settings-shortcut-editor-reset-view = Reset the view to 100%
settings-shortcut-editor-play = Play or pause
settings-shortcut-editor-step-back = Step back a frame
settings-shortcut-editor-step-forward = Step forward a frame
settings-shortcut-editor-jump-back = Step back a second
settings-shortcut-editor-jump-forward = Step forward a second
settings-shortcut-editor-start = Go to the start
settings-shortcut-editor-end = Go to the end
settings-shortcut-editor-split = Split at the playhead
settings-shortcut-editor-snapping = Turn snapping on or off
# Takes the last annotation drawn on the screen away.
settings-shortcut-annotate-undo = Undo
settings-shortcut-annotate-clear = Clear the screen
settings-shortcut-annotate-leave = Stop drawing
# The teleprompter window's keys.
settings-shortcut-teleprompter-play = Play or pause
settings-shortcut-teleprompter-back = Back a few words
settings-shortcut-teleprompter-on = On a few words
settings-shortcut-teleprompter-start = Back to the start
settings-shortcut-teleprompter-larger = Larger text
settings-shortcut-teleprompter-smaller = Smaller text
settings-shortcut-teleprompter-leave = Stop writing, pause, leave the full display, then close

## The Shortcuts pane's keyboard map: a keyboard with each key that does
## something raised. $shortcut and $action name a shortcut, such as Take a
## screenshot; $keys are a combination's keys, drawn as keys or named and
## joined with " + ", such as Command + Shift + S; $key is one key's name.
## $use is what the system or most apps use a combination for, from the
## settings-reserved messages, read after "for".

settings-shortcuts-map = Keyboard
settings-shortcuts-scope = Where the shortcuts work
settings-shortcuts-scope-anywhere = Anywhere
settings-shortcuts-scope-ruler = In Ruler
settings-shortcuts-scope-ocr = In OCR
settings-shortcuts-scope-editor = In Editor
settings-shortcuts-scope-annotate = In Annotate
settings-shortcuts-scope-teleprompter = In Teleprompter
settings-shortcuts-hint = Raised keys do something. Choose a shortcut below, then click a key to give it.
settings-shortcuts-hint-selected = Click a key to give it to “{ $shortcut }”, with the modifiers lit.
# In the scopes whose keys are built in (the editor's and Annotate's), in place of the hints above.
settings-shortcuts-hint-fixed = Raised keys do something. These keys are built in and can't be changed.
# For a key held while dragging a window, such as Glide's.
settings-shortcuts-hint-hold = Click the key to hold for “{ $shortcut }”.
settings-shortcuts-hold = Hold for { $shortcut }
settings-shortcuts-free = Free
settings-shortcuts-reserved-system = { $platform ->
        [windows] Windows
       *[macos] macOS
    } uses { $keys } for { $use }.
settings-shortcuts-reserved-apps = Most apps use { $keys } for { $use }.
settings-shortcuts-assigned = { $shortcut } is now { $keys }.
settings-shortcuts-refused-taken = { $keys } is already used by “{ $action }”.
settings-shortcuts-refused-reserved-system = { $platform ->
        [windows] Windows
       *[macos] macOS
    } uses { $keys } for { $use }. To take them anyway, choose them in the shortcut's field.
settings-shortcuts-refused-reserved-apps = Most apps use { $keys } for { $use }. To take them anyway, choose them in the shortcut's field.
settings-shortcuts-refused-none = Every combination with { $key } is taken.
# Under a shortcut that shares its keys with others. $others names them, joined as a list.
settings-shortcuts-clash = Same keys as { $others }.
settings-shortcuts-clash-remove = Remove from { $shortcut }

## The system's own screenshot keys, such as Windows + Shift + S, which can
## open Screenwide's tools in place of the system's.

settings-shortcuts-system-keys = System keys
settings-shortcuts-system-keys-replace = Use { $platform ->
        [windows] Windows'
       *[macos] macOS's
    } screenshot keys
settings-shortcuts-system-keys-replace-description = They open { -app-name }'s tools instead of the system's.
# Said once on the island, the first time one of the system's keys opens a tool. $keys is the combination, drawn as keys.
settings-system-keys-announce = { $keys } now opens { -app-name }. Turn this off in Settings, under Shortcuts.

## What the system or most apps use a combination for, so a shortcut that
## works anywhere would take it from them. Each is read after "for", as in
## "macOS uses Shift + Command + 3 for screenshots" or "Most apps use
## Command + Z for Undo": an app's menu command keeps its menu's name.

settings-reserved-spotlight = Spotlight
settings-reserved-input-source = switching input sources
settings-reserved-emoji = Emoji & Symbols
settings-reserved-switch-apps = switching apps
settings-reserved-switch-windows = switching between an app's windows
settings-reserved-switch-tabs = switching tabs
settings-reserved-screenshot = screenshots
settings-reserved-lock = locking the screen
settings-reserved-log-out = logging out
settings-reserved-dock = showing and hiding the Dock
settings-reserved-mission-control = Mission Control
settings-reserved-app-windows = showing an app's windows
settings-reserved-spaces = moving between spaces
settings-reserved-force-quit = Force Quit
settings-reserved-close-window = closing windows
settings-reserved-window-menu = the window menu
settings-reserved-start = the Start menu
settings-reserved-task-manager = Task Manager
settings-reserved-security = the security options
settings-reserved-windows-key = its own shortcuts
settings-reserved-quit = Quit
settings-reserved-close = Close
settings-reserved-hide = Hide
settings-reserved-hide-others = Hide Others
settings-reserved-minimize = Minimize
settings-reserved-full-screen = Enter Full Screen
settings-reserved-help = the Help menu
settings-reserved-copy = Copy
settings-reserved-paste = Paste
settings-reserved-cut = Cut
settings-reserved-undo = Undo
settings-reserved-redo = Redo
settings-reserved-select-all = Select All
settings-reserved-save = Save
settings-reserved-find = Find
settings-reserved-new = New
settings-reserved-open = Open
settings-reserved-print = Print
settings-reserved-new-tab = New Tab
settings-reserved-settings = Settings

## Why a shortcut could not be saved, from the app.

settings-error-shortcut-taken = Another { -app-name } shortcut already uses those keys. Choose different keys.
settings-error-shortcut-moment = Those keys already place a moment. Choose different keys.
settings-error-shortcut-invalid = { -app-name } can’t use those keys as a shortcut. Choose different keys.
settings-error-shortcut-unavailable = Another app is already using those keys, so { -app-name } can’t. Choose different keys, or quit that app and try again.
settings-error-ruler-duplicate = Each Ruler shortcut needs its own keys.
settings-error-ocr-duplicate = Each OCR shortcut needs its own keys.
settings-error-moment-duplicate = Each moment needs its own shortcut.

## Search: the field above the sections, its results, and the extra words each
## setting is found by. A keywords message is space-separated words people
## might type for that setting beyond its name; translate them as words a
## reader of the language would try, not word for word.

settings-search-label = Search settings
settings-search-placeholder = Search
settings-search-results = Search results
# Shown above a way into each section when nothing matches. $query is what
# was typed.
settings-search-no-results = No settings match “{ $query }”. Try one of these instead.
settings-search-keywords-folders = location path directory save export
settings-search-keywords-countdown = timer delay wait
settings-search-keywords-frame-rate = fps framerate smooth
settings-search-keywords-glide = window manager snap tile arrange resize
settings-search-keywords-window-gap = gap padding margin spacing
settings-search-keywords-startup = login boot launch open
settings-search-keywords-ruler = measure pixels distance size
settings-search-keywords-annotate = draw pen highlight arrow
settings-search-keywords-ocr = text recognition copy scan
settings-search-keywords-models = ai download machine learning
settings-search-keywords-dictation = voice speech talk type commands vocabulary names spelling
settings-search-keywords-shortcut = hotkey keyboard key binding
settings-search-keywords-updates = version update upgrade
settings-search-keywords-licences = license licence key serial activate buy purchase trial unlock
settings-search-keywords-microphone = mic audio input
settings-search-keywords-moments = marker bookmark flag
settings-search-keywords-auto-zoom = zoom focus follow pointer cursor
settings-search-keywords-anti-flicker = flicker banding lights 50hz pal webcam
settings-search-keywords-bars = crop menubar taskbar dock bar top bottom edge

## The pictures a setting's preview is drawn on: a sample desktop the tools
## work on. OCR reads the note back, so it is set in the app's language.

# The first line of a note on the sample desktop, in bold.
settings-preview-note-title = Launch checklist
settings-preview-note-first = Record the demo at 60 fps
settings-preview-note-second = Ship on Thursday
# The title of a sample app window Annotate points at.
settings-preview-window-title = Publish changes
# The button in that sample window.
settings-preview-window-button = Publish
# The heading of a sample dashboard with a chart.
settings-preview-dashboard-title = Weekly views
# Folders and a file on the sample desktop, shown while Hide desktop icons is off.
settings-preview-desktop-projects = Projects
settings-preview-desktop-screenshots = Screenshots
settings-preview-desktop-notes = notes.txt

## Licences

settings-licences-references = References
settings-licences-teleprompter = Teleprompter
# A paid tool's row, when a key for it is held.
settings-licences-licensed = Licensed
# A paid tool's row during the trial. $days is the whole days left, at least 1.
settings-licences-trial =
    { $days ->
        [one] In trial, 1 day left
       *[other] In trial, { $days } days left
    }
# A paid tool's row before the tool has been used. $days is the trial's length.
settings-licences-trial-unused = Free for { $days } days from the first time you use it
settings-licences-expired = Trial ended. Buy it or add its key below.
settings-licences-buy = Buy
# The Buy button's name for assistive technology. $tool is the tool's name.
settings-licences-buy-tool = Buy { $tool }
settings-licences-bundle = Bundle
settings-licences-bundle-title = Every paid tool
settings-licences-bundle-description = Includes every paid tool added later.
settings-licences-buy-bundle = Buy bundle
settings-licences-keys = Keys
settings-licences-keys-description = Each tool has its own key, sent with your receipt.
settings-licences-add-key = Licence key
settings-licences-key-placeholder = Paste a key from your receipt
settings-licences-add = Add
# A paid tool's row when its key is held. $hint is the key's last characters, so a buyer's keys can be told apart.
settings-licences-licensed-key = Licensed, key ending { $hint }
# $tools is a list of tool names, such as "Ruler and Glide".
settings-licences-key-added = Key unlocked { $tools }.
settings-licences-key-invalid = This key isn’t recognised. Check it against your receipt.
settings-licences-key-disabled = This key has been refunded or turned off.
settings-licences-key-unavailable = Couldn’t reach the licence server. Check your connection and try again.
settings-licences-remove = Remove the key ending { $hint }
settings-licences-confirm-remove = Confirm removing the key ending { $hint }

## Tools' Dictation section. Each command is named here for what it does; what to say comes from the language dictation is in, which may not be the app's.

settings-dictation-commands = Spoken commands
settings-dictation-commands-description = Say a command on its own, with a pause before and after.
settings-dictation-command-scratch = Take back the last sentence
settings-dictation-command-new-line = New line
settings-dictation-command-new-paragraph = New paragraph
# Ends a numbered list, so what follows starts a new paragraph instead of running on in its last point.
settings-dictation-command-end-list = End a list
settings-dictation-command-punctuation = Punctuation
settings-dictation-command-add-word = Add a word
settings-dictation-command-remove-word = Remove a word
# A phrase to say, in the language dictation is in, quoted.
settings-dictation-command-phrase = “{ $phrase }”
# What to say for a command. $phrases are the quoted phrases that do it, as a list of alternatives.
settings-dictation-command-say = Say { $phrases }.
settings-dictation-command-say-word = Say { $phrases }, then spell or say the word.
# The picture's sample of dictating, as the island shows it while the words are said. The commands said in it come from the language dictation is in.
settings-dictation-sample-before = Let's meet on Thursday.
settings-dictation-sample-after = Let's meet on Friday.
settings-dictation-sample-greeting = Dear Sam,
settings-dictation-sample-thanks = Thanks for coming to the review.
settings-dictation-sample-question = Can you make it on Friday
# The two points of a numbered list, each written after its number.
settings-dictation-sample-list-first = Book the room.
settings-dictation-sample-list-second = Send the agenda.
settings-dictation-sample-word = I recorded the demo with Screenwide.
# A word spelled out letter by letter, and the word it adds or removes.
settings-dictation-sample-added-spelled = S C R E E N W I D E
settings-dictation-sample-added = Screenwide
settings-dictation-sample-removed-spelled = O V E R P O L I S H
settings-dictation-sample-removed = Overpolish
