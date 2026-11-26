# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Meeting notes: a call's speech written out into a Markdown file, keeping no audio.

## The island while notes are taken

notes-island = Notes
# The notebook glyph's name while notes are taken, read aloud by screen readers.
notes-island-taking = Taking notes
# The time the notes have run, read aloud by screen readers.
notes-island-elapsed = Taking notes for { $time }
# Stops the notes and saves them.
notes-island-done = Done
# Stops the notes and throws them away, to Recently Deleted.
notes-island-discard = Discard notes
notes-island-confirm-discard = Confirm discarding
# Under the row while the speech models load as notes start. What is said meanwhile is kept and written once they have loaded.
notes-island-loading-model = Loading model
# Shown while the last lines are written and the voices told apart.
notes-island-finishing = Telling the speakers apart
notes-island-saved = Notes saved
notes-island-open = Show Notes
notes-island-close = Close
notes-island-open-settings = Open Settings

## Why notes could not be taken

notes-island-no-model = Download the Meeting notes models in Settings to take notes.
notes-island-no-microphone = Screenwide needs access to the microphone to take notes.
notes-island-no-microphone-connected = No microphone is connected. Connect one to take notes.
notes-island-no-system-audio = Screenwide needs Screen Recording access to hear the call.
notes-island-not-started = The notes could not start. Try again.
notes-island-failed = Notes stopped. What was written so far is saved.

## The notes in the projects window

notes-view-back = Projects
notes-view-copy-path = Copy Path
# Shown while the notes are still being taken, so they cannot be edited yet.
notes-view-still-taking = These notes are still being taken. Stop them to edit.
# A speaker's name, edited in place to rename them throughout the notes.
notes-view-speaker-name = Name for { $speaker } throughout the notes
# The speakers across the head of the notes, for screen readers.
notes-view-speakers = Speakers
# A speaker's share of the talking, a toggle that shows only what the chosen
# speakers said. $share is a percentage.
notes-view-speaker-filter = Show only what { $speaker } said, { $share } of the talking
# In place of other speakers' paragraphs while some speakers are chosen; shows them.
notes-view-hidden =
    { $count ->
        [one] 1 paragraph hidden
       *[other] { $count } paragraphs hidden
    }
# Plays a few seconds of a speaker not yet named, to tell who they are.
notes-view-speaker-hear = Hear { $speaker }
# Under the speakers while any can still be heard.
notes-view-voices-kept = Each unnamed speaker's voice is kept for a week, or until you name them, so you can tell who they are.
# The button beside a speaker's name that gives what they said there to someone else.
notes-view-said-by = Change who said this, now { $speaker }
# The heading of the speakers to give a run of turns to.
notes-view-said-by-menu = Said by
# A turn's words, for screen readers.
notes-view-turn = What { $speaker } said
# A paragraph with no speaker, for screen readers.
notes-view-paragraph = Paragraph
notes-view-save-failed = Could not save the notes.
notes-view-undo = Undo
notes-view-redo = Redo
# The button beside a speaker's name that removes what they said there.
notes-view-remove-run = Remove what { $speaker } said here
# The search at the top of the notes, over what was said.
notes-view-search = Search notes
# Beside the search: opens and closes Replace under it.
notes-view-find = Find and Replace
notes-view-replace-field = Replace with
# Replaces the match gone to with what was typed.
notes-view-replace = Replace
notes-view-replace-all = Replace All

## What may be private in the notes

# Over a paragraph, the badges naming what in it may be private, for screen readers.
notes-sensitive-found = May be private
# A badge over a paragraph naming what in it may be private, and how many; opens what can be done about them. Also heads that menu.
notes-sensitive-kind =
    { $kind ->
        [name]
            { $count ->
                [one] Name
               *[other] { $count } names
            }
        [company]
            { $count ->
                [one] Company
               *[other] { $count } companies
            }
        [address]
            { $count ->
                [one] Address
               *[other] { $count } addresses
            }
        [email]
            { $count ->
                [one] Email
               *[other] { $count } emails
            }
        [phone]
            { $count ->
                [one] Phone number
               *[other] { $count } phone numbers
            }
        [account]
            { $count ->
                [one] Account number
               *[other] { $count } account numbers
            }
        [card]
            { $count ->
                [one] Card number
               *[other] { $count } card numbers
            }
        [link]
            { $count ->
                [one] Link
               *[other] { $count } links
            }
        [health]
            { $count ->
                [one] Health
               *[other] { $count } health mentions
            }
        [religion]
            { $count ->
                [one] Religion
               *[other] { $count } religion mentions
            }
        [sexuality]
            { $count ->
                [one] Sexuality
               *[other] { $count } sexuality mentions
            }
        [ethnicity]
            { $count ->
                [one] Ethnicity
               *[other] { $count } ethnicity mentions
            }
        [politics]
            { $count ->
                [one] Politics
               *[other] { $count } political mentions
            }
       *[union]
            { $count ->
                [one] Union membership
               *[other] { $count } union mentions
            }
    }
# The menu a badge or a marked word opens, for screen readers.
notes-sensitive-menu = What may be private
# In a badge's menu: puts REDACTED in place of what the badge names, in this paragraph alone.
notes-sensitive-redact-found =
    { $count ->
        [one] Redact it here
       *[other] Redact them here
    }
# In a marked word's menu: puts REDACTED in place of these words in this paragraph alone.
notes-sensitive-redact-here = Redact here
# In a marked word's menu: puts REDACTED in place of these words wherever they are said in the notes.
notes-sensitive-redact-throughout = Redact throughout the notes
notes-sensitive-delete-paragraph = Delete paragraph
# In a badge's or a marked word's menu: these words are not private, so they are no longer pointed out anywhere in these notes.
notes-sensitive-not-private = Not private
