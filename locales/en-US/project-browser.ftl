# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The project browser: the window listing recordings and screenshots.

project-browser-title = Projects
project-browser-logo = { -app-name }
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

## Empty states

project-browser-empty-title = No projects
project-browser-empty-recent = Recordings and screenshots you make or open appear here.
project-browser-empty-location = Projects saved in this folder appear here.
project-browser-empty-deleted-title = No deleted projects
project-browser-empty-deleted = Projects you delete wait here for 30 days.
project-browser-no-results-title = No results
project-browser-no-results = No project names contain “{ $query }”.

## Toolbar above the projects

project-browser-select-all = Select all
project-browser-deselect-all = Deselect all
# The filter choosing which kind of project is listed.
project-browser-kind = Kind
project-browser-kind-all = All
project-browser-kind-screen = Screen
project-browser-kind-camera = Camera
project-browser-kind-audio = Audio
project-browser-kind-screenshot = Screenshots
project-browser-selected-count = { $count } selected
project-browser-project-count =
    { $count ->
        [one] { $count } project
       *[other] { $count } projects
    }

## Project list

project-browser-search-results = Search results
# Group headings, newest first. Older projects are grouped by month.
project-browser-group-today = Today
project-browser-group-yesterday = Yesterday
project-browser-group-previous-week = Previous 7 Days
project-browser-group-previous-month = Previous 30 Days
project-browser-group-unknown = Unknown Date
# How long a project in Recently Deleted has before it goes to the Trash.
project-browser-days-left =
    { $days ->
        [one] { $days } day left
       *[other] { $days } days left
    }

## Project card

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
project-browser-move-to = Move To
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
project-browser-empty-recently-deleted-confirm = Confirm emptying Recently Deleted

## Notices below the projects

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
