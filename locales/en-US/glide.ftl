# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Glide: moving a window into a region of the screen, another Space or
### another display with a gesture. These name the Glide island and the
### picker's cards for screen readers.

# $region is one of the region names below.
glide-destination = Glide destination: { $region }
glide-no-destination = No Glide destination
glide-minimize = Minimize
glide-locked = This window cannot be moved

## Regions of the screen a window can be placed in

glide-region-full-screen = full screen
glide-region-left-half = left half
glide-region-right-half = right half
glide-region-left-two-thirds = left two thirds
glide-region-right-two-thirds = right two thirds
glide-region-left-third = left third
glide-region-middle-third = middle third
glide-region-right-third = right third
glide-region-top-half = top half
glide-region-bottom-half = bottom half
# A column region limited to one row, such as "left third, top half".
glide-region-part = { $columns }, { $rows }

## Moving a window to another Space or display

# $index is the chosen Space's place in the row, counted from 1, and $count how
# many Spaces there are.
glide-picker-space = Moving to Space { $index } of { $count }
# $index is the chosen display's place, counted from 1, and $count how many
# displays there are.
glide-picker-display = Moving to display { $index } of { $count }
