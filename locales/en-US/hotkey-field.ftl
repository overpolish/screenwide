# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The field that records a keyboard shortcut. Most of these are read out
### by screen readers while a shortcut is being recorded.

hotkey-field-instructions-single = Press a key or hold and release an auxiliary mouse button. Escape cancels; Tab leaves.
hotkey-field-instructions-local = Press a key or use a modifier and a key. Escape cancels.
hotkey-field-instructions = Press a key, then choose its combination, or press the whole shortcut. Escape cancels; Tab leaves.
hotkey-field-instructions-clearable = Press a key, then choose its combination, or press the whole shortcut. Escape cancels; Tab leaves. Delete clears.
hotkey-field-updated = Shortcut updated.
hotkey-field-cancelled = Shortcut capture cancelled.
hotkey-field-cleared = Shortcut cleared.
hotkey-field-reset-done = Shortcut reset.
hotkey-field-hold-error = Hold the button for half a second, then release.
# $error says what went wrong, in English.
hotkey-field-capture-failed = Could not start capture: { $error }
# $label names the shortcut; $keys are its keys joined with " + ".
hotkey-field-value = { $label }: { $keys }
hotkey-field-not-set = Not set
hotkey-field-preparing = Preparing
hotkey-field-testing-hold = Testing button hold
hotkey-field-keep-holding = Keep holding
hotkey-field-press-key-or-button = Press key or hold button
hotkey-field-press-shortcut = Press a key
hotkey-field-set = Set shortcut
# $label names the shortcut.
hotkey-field-clear = Clear { $label }
hotkey-field-reset = Reset { $label }
# A key that only works while a tool is open, pressed when another of that tool's shortcuts has it. $keys are its keys joined with " + "; $action names the other shortcut.
hotkey-field-local-taken = { $keys } is already used by “{ $action }”. Press another key.

## A key pressed alone, and the combinations of it with modifiers offered
## as chips to choose from. $key is the key's name, such as S; $label names
## the shortcut being set; $keys are a combination's keys joined with " + ";
## $action names another shortcut; $use is what the system or most apps use
## a combination for, read after "for", such as screenshots.

hotkey-field-pick-start = Choose a combination for { $key }. Arrow keys move; Return takes it.
# The modifier toggles beside the key.
hotkey-field-modifiers = { $label }: modifiers for { $key }
hotkey-field-suggestions = Combinations for { $key }
hotkey-field-pick-free = Return takes it; Escape cancels.
hotkey-field-pick-taken = Already used by “{ $action }”.
hotkey-field-pick-reserved-system = { $platform ->
        [windows] Windows
       *[macos] macOS
    } uses { $keys } for { $use }. Return or a second click takes them anyway.
hotkey-field-pick-reserved-apps = Most apps use { $keys } for { $use }. Return or a second click takes them anyway.
# A shortcut that works anywhere needs a modifier besides Shift, or it would type.
hotkey-field-pick-modifier = Add { $platform ->
        [windows] Ctrl or Alt
       *[macos] Command, Option or Control
    } so { $key } still types.
hotkey-field-chip-taken = { $keys }, already used by “{ $action }”
hotkey-field-chip-reserved-system = { $keys }, which { $platform ->
        [windows] Windows
       *[macos] macOS
    } uses for { $use }
hotkey-field-chip-reserved-apps = { $keys }, which most apps use for { $use }
