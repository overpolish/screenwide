# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Dictation: speech typed into any app as it is spoken.

## The island while dictating

dictation-island = Dictation
# The row's title while the microphone is heard.
dictation-island-listening = Listening
# Before any words are heard. { $key } is the Escape key, drawn as a keycap.
dictation-island-hint = Speak, and the words go where your cursor is. { $key } finishes.
# Before any words are heard, in a language whose text is written at each pause rather than as it is spoken. { $key } is the Escape key, drawn as a keycap.
dictation-island-hint-at-pauses = Speak, and the text appears each time you pause. { $key } finishes.
# Stops listening and types what was heard.
dictation-island-done = Done
dictation-island-cancel = Cancel dictation
# Shown once Cancel dictation is pressed: pressing again throws away what was heard.
dictation-island-confirm-cancel = Confirm discarding
# The row's title once listening stops, while the last words are heard and typed.
dictation-island-typing = Typing
dictation-island-close = Close
dictation-island-open-settings = Open Settings

## Words added to or removed from the word list by saying "add word" or "remove word". $word is the word, as spelled.

dictation-island-word-added = Added “{ $word }” to your words.
dictation-island-word-already-listed = “{ $word }” is already in your words.
dictation-island-word-removed = Removed “{ $word }” from your words.
dictation-island-word-not-listed = “{ $word }” is not in your words.
dictation-island-word-failed = Could not change your words for “{ $word }”.

## Why dictation could not start or type

dictation-island-no-model = Download the Dictation model in Settings to dictate.
dictation-island-no-microphone = Screenwide needs access to the microphone to dictate.
dictation-island-no-accessibility = Screenwide needs Accessibility access to type for you.
dictation-island-password-field = Dictation does not type into password fields.
# { $key } is the paste shortcut, drawn as keycaps.
dictation-island-administrator = Windows does not let Screenwide type into apps running as administrator. Dictation moved to clipboard. Use { $key } to paste it.
# Nothing that takes text had the keyboard when dictation pasted. { $key } is the paste shortcut, drawn as keycaps.
dictation-island-on-clipboard = Couldn't insert your dictation into the focused app. Dictation moved to clipboard. Use { $key } to paste it.
dictation-island-failed = Dictation stopped. Try again.
