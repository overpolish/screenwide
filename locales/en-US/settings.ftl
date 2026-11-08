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
# The Shortcuts pane's group for Keep Awake, the tray's switch that keeps the computer awake and its user shown as active.
settings-keep-awake = Keep Awake

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
# Under the app's name in a copy installed from the Microsoft Store, which updates it. $version is its version, such as 1.2.0.
settings-version-store = Version { $version }. Updates come from the Microsoft Store.
settings-update-to = Update to v{ $version }
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
settings-glide-haptics = Trackpad feedback
settings-glide-haptics-description = Feel a tap when the next move is ready.
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

## Models: transcription, Studio sound, Studio ducking and background removal. $model is a model's name.

settings-transcription-local = Models run on this computer. Recordings are never uploaded.
# The model the microphone's Studio sound switch uses, named for what it does.
settings-transcription-studio-sound = Studio sound
settings-transcription-studio-sound-description = Enhances your voice.
# The model the microphone's Studio ducking switch uses, named for what it does.
settings-transcription-studio-ducking = Studio ducking
settings-transcription-studio-ducking-description = Keeps your voice clear over other audio.
# The model the camera's Remove background switch uses, named for what it does.
settings-transcription-camera-background = Background removal
settings-transcription-camera-background-description = Removes the background behind you on camera.
settings-transcription-camera-objects = Camera objects
settings-transcription-camera-objects-description = Keeps or removes objects on camera.
# The model the recording's transcript and voice notes are made with, named for what it does.
settings-transcription-transcripts = Transcripts
settings-transcription-transcripts-description = Turns your recordings and voice notes into text.
settings-transcription-language = Language
settings-transcription-language-description = The language your recordings are spoken in.
# $language is the computer's own language.
settings-transcription-system-language = System ({ $language })
settings-transcription-detect = Detect Automatically
settings-transcription-download = Download
settings-transcription-cancel = Cancel
settings-transcription-downloading = Downloading the { $model } model
settings-transcription-remove = Remove the { $model } model
settings-transcription-confirm-remove = Confirm removing the { $model } model

## Keyboard shortcuts

settings-shortcut-toggle-controls = Show or hide recording controls
settings-shortcut-start-stop = Start or stop recording
settings-shortcut-pause-resume = Pause or resume recording
settings-shortcut-save-replay = Save replay
settings-shortcut-save-replay-description = Works while the replay buffer is on.
settings-shortcut-screenshot = Take a screenshot
settings-shortcut-screenshot-description = Choose an area of your screen to capture.
settings-shortcut-copy-screenshot = Copy a screenshot
settings-shortcut-copy-screenshot-description = Choose an area to copy, ready to paste elsewhere.
settings-shortcut-reference-screenshot = Keep a reference
settings-shortcut-reference-screenshot-description = Choose an area to keep on screen at its true size.
settings-shortcut-keep-awake = Turn Keep Awake on or off
settings-shortcut-keep-awake-description = Keeps the computer awake and shows you as active.
# The row of a moment's shortcut in the Shortcuts pane. $moment is the moment's name, which the user chooses.
settings-shortcut-place-moment = Place { $moment }

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
settings-shortcuts-hint = Raised keys do something. Choose a shortcut below, then click a key to give it.
settings-shortcuts-hint-selected = Click a key to give it to “{ $shortcut }”, with the modifiers lit.
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
# A paid tool's row, when a key for it is held.
settings-licences-licensed = Licensed
# A paid tool's row during the trial. $days is the whole days left, at least 1.
settings-licences-trial =
    { $days ->
        [one] In trial, 1 day left
       *[other] In trial, { $days } days left
    }
# A paid tool's row before any paid tool has been used. $days is the trial's length.
settings-licences-trial-unused = Free for { $days } days from the first time you use a paid tool
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
