# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Annotations: the arrows, counters, highlights, shapes and the rest drawn
### over a screenshot, a recording or the desktop. The editor and the live
### Annotate overlay share these, so a tool reads the same in both.

## Drawing tools, by name, then by what they do

annotation-tool-arrow = Arrow
annotation-tool-counter = Counter
annotation-tool-draw = Draw
annotation-tool-highlight = Highlight
annotation-tool-image = Image
annotation-tool-magnify = Magnifier
annotation-tool-redact = Redact
annotation-tool-shape = Shape
annotation-tool-spotlight = Spotlight
annotation-tool-focus = Focus
annotation-tool-text = Text
annotation-tool-arrow-name = Draw an arrow
annotation-tool-counter-name = Drop a counter
annotation-tool-draw-name = Draw freehand
annotation-tool-highlight-name = Highlight text
annotation-tool-image-name = Place an image
annotation-tool-magnify-name = Magnify a detail
annotation-tool-redact-name = Redact a region
annotation-tool-shape-name = Draw a shape
annotation-tool-spotlight-name = Spotlight a region
annotation-tool-focus-name = Keep a region sharp
annotation-tool-text-name = Add text

## What the size control measures

annotation-size-width = Width
annotation-size-size = Size
annotation-size-height = Height
annotation-size-block = Block size

## Colours

annotation-color = Colour
annotation-custom-color = Custom colour
annotation-saved-color-armed = Right-click or press Delete again to remove { $color }
annotation-color-red = Red
annotation-color-orange = Orange
annotation-color-yellow = Yellow
annotation-color-green = Green
annotation-color-blue = Blue
annotation-color-purple = Purple
annotation-color-white = White
annotation-color-black = Black

## Style controls

annotation-alignment = Alignment
annotation-align-left = Left
annotation-align-center = Centre
annotation-align-right = Right
annotation-angle = Angle
annotation-angle-value = Angle value
annotation-radius = Radius
annotation-randomise = Randomise
# How wide a blurred redaction's blur spreads, in steps from lightest to strongest.
annotation-blur-strength = Strength
# One of a blur's steps, named by its place from the lightest and how many there are.
annotation-blur-strength-step = Strength { $strength } of { $count }
# A toggle: whether an image casts a shadow onto the picture under it.
annotation-shadow-drop = Drop shadow
# A toggle: whether a recording gives a placed image a slight, slow turn and drift. Each time it is turned on the sway is new.
annotation-sway-on = Sway
# A toggle: whether a recording loops a moving image for its whole clip rather than playing it through once.
annotation-playback-loop = Loop
# Whether what lies outside a spotlight is blurred.
annotation-outside-blur = Blur
# Whether a highlight fits itself to the text under it or is laid by hand.
annotation-fit = Fit
annotation-fit-auto = Auto
annotation-fit-manual = Manual
# A toggle: whether a highlight covers the box dragged rather than fitting the text under it.
annotation-fit-box = Cover the box
# Which ends of an arrow carry a head.
annotation-head = Head
annotation-head-none = None
annotation-head-end = End
annotation-head-both = Both
# Whether a highlight recolours the text it covers or tints it like a marker.
annotation-ink = Ink
annotation-ink-recolor = Recolor
annotation-ink-tint = Tint
# A toggle: whether a highlight or a shape is drawn by hand rather than clean. Each time it is turned on the wobble is new.
annotation-stroke-hand-drawn = Hand-drawn
annotation-redaction = Redaction
annotation-redaction-erase = Erase
annotation-redaction-color = Colour
annotation-redaction-pixelate = Pixelate
annotation-redaction-blur = Blur
annotation-redaction-shuffle-pixelation = Shuffle pixelation
annotation-redaction-shuffle-blur = Shuffle blur
# Secure pixelation lays its blocks out at random; classic averages each one.
annotation-pixelation = Pixelation
annotation-pixelation-secure = Secure
annotation-pixelation-classic = Classic
# A toggle: whether a pixelated redaction uses classic pixelation, only a look, rather than secure.
annotation-pixelation-classic-on = Classic pixelation
