# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The project browser: the window listing recordings and screenshots.

project-browser-search-label = Search projects
project-browser-search-placeholder = Search
project-browser-open-file = Open File

## Sidebar

project-browser-locations = Project locations
project-browser-recent = Recent
project-browser-recently-deleted = Recently Deleted
project-browser-add-location = Add Location
# The title of the folder picker Add Location opens.
project-browser-add-location-dialog = Add Location
project-browser-location-actions = Location actions
# $platform is windows or macos: the file manager is Explorer or Finder.
project-browser-open-in-file-manager =
    { $platform ->
        [windows] Open in Explorer
       *[other] Open in Finder
    }
project-browser-remove-location = Remove Location
# The strip at the sidebar's foot drawing the disk each project takes, one
# segment per project.
project-browser-storage = Storage
# A segment of the storage strip, which goes to the project's card. $size is
# what the project takes on disk, such as "1.2 GB".
project-browser-go-to-project = Go to { $title }, { $size }

## Empty states

project-browser-empty-title = No projects
project-browser-empty-deleted-title = No recently deleted projects
project-browser-no-results-title = No results

## Toolbar above the projects

# The filter choosing which kind of project is listed.
project-browser-kind = Kind
project-browser-kind-all = All
project-browser-kind-screen = Screen
project-browser-kind-camera = Camera
project-browser-kind-audio = Audio
project-browser-kind-screenshot = Screenshots
project-browser-project-count =
    { $count ->
        [one] { $count } project
       *[other] { $count } projects
    }
project-browser-loading = Loading projects

## Search

# Above a search's results: how many projects something was found in.
project-browser-search-found =
    { $count ->
        [one] Found in { $count } project
       *[other] Found in { $count } projects
    }
# Beside a project a search found: how many moments in it match, or none when
# only its name does.
project-browser-search-finds =
    { $count ->
        [0] Found by its name
        [one] { $count } moment
       *[other] { $count } moments
    }
# The list of a project's moments that match a search, for assistive technology.
project-browser-search-finds-in = Moments in { $title }

## Search by date

# Words the search reads as when something was recorded. Each message is a
# list of the words or phrases that mean one thing, separated by commas, as
# in "last, past". A phrase may be several words ("last week"). They are
# matched without regard to case or accents, and English stays understood
# beside them. Weekday and month names come from the language's calendar and
# need no list.
project-browser-date-words-today = today
project-browser-date-words-yesterday = yesterday
project-browser-date-words-this-week = this week
project-browser-date-words-last-week = last week
project-browser-date-words-this-month = this month
project-browser-date-words-last-month = last month
# Before a number of hours or minutes: "last 2 hours", "past hour".
project-browser-date-words-last = last, past
# After a number of hours or minutes ("2 hours ago"), or before it in a
# language that puts it first.
project-browser-date-words-ago = ago
# Standing for one before "hour" or "minute": "an hour ago".
project-browser-date-words-one = a, an, one
project-browser-date-words-hours = hour, hours, hr, hrs, h
project-browser-date-words-minutes = minute, minutes, min, mins, m
project-browser-date-words-morning = morning
project-browser-date-words-afternoon = afternoon
project-browser-date-words-evening = evening
project-browser-date-words-night = night
# After an hour, as in "3 pm". Stops are ignored, so "a.m." reads as "am".
project-browser-date-words-am = am
project-browser-date-words-pm = pm
# Above the projects a search by date found: when it read them as recorded.
project-browser-date-today = Recorded today
project-browser-date-yesterday = Recorded yesterday
# $date is a day ("Monday, Oct 27"), perhaps with an hour of it
# ("Monday, Oct 27, 3 – 4 PM").
project-browser-date-on = Recorded { $date }
# $hours is an hour of the day, as "3 – 4 PM".
project-browser-date-today-hours = Recorded today, { $hours }
project-browser-date-yesterday-hours = Recorded yesterday, { $hours }
# A part of a day: today's, yesterday's, or another's, $date as in
# project-browser-date-on ("Monday, Oct 27"). Night runs from 9 PM to 5 AM.
project-browser-date-part =
    { $day ->
        [today]
            { $part ->
                [morning] Recorded this morning
                [afternoon] Recorded this afternoon
                [evening] Recorded this evening
               *[night] Recorded tonight
            }
        [yesterday]
            { $part ->
                [morning] Recorded yesterday morning
                [afternoon] Recorded yesterday afternoon
                [evening] Recorded yesterday evening
               *[night] Recorded last night
            }
       *[other]
            { $part ->
                [morning] Recorded { $date } in the morning
                [afternoon] Recorded { $date } in the afternoon
                [evening] Recorded { $date } in the evening
               *[night] Recorded { $date } at night
            }
    }
# $period is a month ("October", "October 2025") or a year ("2025").
project-browser-date-in = Recorded in { $period }
project-browser-date-this-week = Recorded this week
project-browser-date-last-week = Recorded last week
project-browser-date-this-month = Recorded this month
project-browser-date-last-month = Recorded last month
project-browser-date-recent =
    { $unit ->
        [hour]
            { $count ->
                [one] Recorded in the last hour
               *[other] Recorded in the last { $count } hours
            }
       *[minute]
            { $count ->
                [one] Recorded in the last minute
               *[other] Recorded in the last { $count } minutes
            }
    }

## Project list

# Group headings, newest recorded first. Older projects are grouped by month.
project-browser-group-today = Today
project-browser-group-yesterday = Yesterday
project-browser-group-previous-week = Previous 7 Days
project-browser-group-unknown = Unknown Date
# How long a project in Recently Deleted has before it goes to the Trash.
project-browser-days-left =
    { $days ->
        [one] { $days } day left
       *[other] { $days } days left
    }

## Project card

# The card's picture, which opens the project in the editor.
project-browser-open-project = Open { $title }
project-browser-select-project = Select { $title }
project-browser-more-actions = More actions for { $title }
project-browser-project-name = Project name
# A rename to a name no file can have, such as one made only of dots.
project-browser-name-unusable = That name cannot be used
project-browser-unavailable = Not available
project-browser-badge-screenshot = Screenshot
project-browser-badge-replay = Replay
# What a recording holds, read out after "Open <name>" in a list.
project-browser-track-screen = screen
project-browser-track-camera = camera
project-browser-track-system-audio = system audio
project-browser-track-microphone = microphone

## Actions on projects

project-browser-project-actions = Project actions
project-browser-restore = Restore
# $platform is windows or macos: deleted files go to the Recycle Bin or the
# Trash.
project-browser-move-to-trash =
    { $platform ->
        [windows] Move to Recycle Bin
       *[other] Move to Trash
    }
# A button and a menu section listing the folders to move projects to.
project-browser-move-to = Move to
project-browser-delete = Delete
project-browser-duplicate = Duplicate
# $platform is windows or macos: the file manager is Explorer or Finder.
project-browser-show-in-file-manager =
    { $platform ->
        [windows] Show in Explorer
       *[other] Show in Finder
    }
project-browser-remove-from-recent = Remove from Recent
# Sends everything in Recently Deleted on to the Trash, after a confirmation.
project-browser-empty-recently-deleted = Empty
# The same button once pressed, asking for a second press. Short: the button
# keeps the width of the longer of the two labels.
project-browser-empty-recently-deleted-confirm = Empty Now

## The island at the foot of the window: the choice of projects, and what is
## passing, such as a copy under way or a delete that can be undone

project-browser-island = Selection and progress
project-browser-selected-count = { $count } selected
project-browser-select-all = Select all
project-browser-deselect-all = Deselect all
# [1] names the single project as $title; the other variants count them.
project-browser-duplicating =
    { $count ->
        [1] Duplicating “{ $title }”
       *[other] Duplicating { $count } projects
    }
# $destination is the name of the folder the projects go to.
project-browser-moving =
    { $count ->
        [1] Moving “{ $title }” to { $destination }
       *[other] Moving { $count } projects to { $destination }
    }
# A move to a folder whose name is not known.
project-browser-moving-elsewhere =
    { $count ->
        [1] Moving “{ $title }” to another location
       *[other] Moving { $count } projects to another location
    }
project-browser-deleted =
    { $count ->
        [1] Moved “{ $title }” to Recently Deleted.
       *[other] Moved { $count } projects to Recently Deleted.
    }
project-browser-undo = Undo
# A project that would not open; $reason is one of the sentences below.
project-browser-open-failed = Project could not open: { $reason }
project-browser-open-failed-discarded = This recording was discarded.
project-browser-open-failed-gone = This project could not be found.
project-browser-open-failed-unreadable = This project could not be read.
project-browser-open-failed-not-project = This file is not a Screenwide project.
project-browser-open-failed-newer = This project was made by a newer version of Screenwide.
project-browser-open-failed-damaged = This project’s file is damaged.
project-browser-open-failed-recording-missing = The recording is missing from this project.
project-browser-open-failed-recording-unplayable = The recording in this project cannot be played.
project-browser-open-failed-picture-unreadable = A picture in this project could not be read.
# Anything else that went wrong; the details are in the log.
project-browser-open-failed-other = Something went wrong while opening it.
