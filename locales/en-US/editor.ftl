# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The editor window: its title bar, tools, preview and menus.

editor-opening = Opening the editor
editor-screenshot-preview = Screenshot preview
editor-copy = Copy
# The title bar's command in a recording: copies the frame under the playhead, as the preview draws it.
editor-copy-frame = Copy Frame
editor-export = Export
editor-delete = Delete
editor-confirm-delete = Confirm deleting
# The screenshot editor's title bar command while the screenshot is not yet a project: keeps it in Projects.
editor-save-screenshot = Save

## Asked on closing a screenshot that is not in Projects and holds work not copied or exported

editor-unsaved-screenshot-title = Save this screenshot?
editor-unsaved-screenshot-message = It is not in Projects yet, so closing it without saving deletes it.
editor-unsaved-screenshot-save = Save
editor-unsaved-screenshot-discard = Don’t Save
editor-unsaved-screenshot-cancel = Cancel

## The tool column

editor-toolbar-more-tools = More tools
editor-toolbar-more-annotation-tools = More annotation tools
# Names the editor's tool column, Select and the pens, for assistive technology.
editor-toolbar-tools = Tools
# Names the editor's modes in its tool column, each opening an island for the whole recording, for assistive technology.
editor-toolbar-modes = Modes
# The mode that opens the script on its own, to cut the recording by its words.
editor-toolbar-transcript = Transcript
editor-tool-select = Select
editor-tool-select-name = Select screenshot
editor-tool-select-clip-name = Select recording clip
# The crop of the layer chosen, the screen or the camera of a recording, or a screenshot: its island's title and its key in the Shortcuts pane.
editor-tool-crop = Crop
editor-tool-frame = Frame

## Menus

editor-annotation-actions = Annotation actions
editor-layer-actions = Layer actions
# Moves through the drawing order, as video layers are moved.
editor-arrange = Arrange
editor-arrange-forward = Move Forward
editor-arrange-front = Move to Front
editor-arrange-backward = Move Backward
editor-arrange-back = Move to Back

## The recording preview

editor-recording-preview = Recording preview
editor-recording-composed-preview = Composed recording preview
editor-recording-screen-preview = Screen preview
editor-recording-camera-preview = Camera preview
editor-recording-preparing-preview = Preparing the preview
editor-recording-preparing-recording = Preparing recording preview

## Scenes, as the lane and the Scene panel name them

editor-scene-blank = Blank
editor-scene-custom = Custom
editor-scene-camera-only = Camera only
editor-scene-picture-in-picture = Picture in picture
editor-scene-screen-only = Screen only
editor-scene-side-by-side = Side by side
editor-scene-stacked = Stacked

## annotation.

editor-pin-pin = Pin to Content
editor-pin-unpin = Unpin
editor-pin-hide = Content Out of View
editor-pin-show = Content Back in View
editor-pin-clear-corrections = Clear Corrections
editor-pin-tracking-section = Tracking

## Time left on a running export

editor-eta-under-minute = Less than a minute remaining
# $duration is a list of the parts below, such as "1 hr 5 min".
editor-eta = About { $duration } remaining
editor-eta-days =
    { $count ->
        [one] { $count } day
       *[other] { $count } days
    }
editor-eta-hours = { $count } hr
editor-eta-minutes = { $count } min

## Your caption templates, in and out of the app. The reason stays in English.

editor-template-import-failed = Template could not be imported: { $reason }
editor-template-export-failed = Template could not be exported: { $reason }
editor-template-remove-failed = Template could not be removed: { $reason }

## A short-video app's interface laid over the picture while editing, never exported. Sample words a post in that app shows; the app names its own tabs.

editor-overlay-caption = What your viewers read under the video #tutorial
editor-overlay-dislike = Dislike
editor-overlay-follow = Follow
editor-overlay-following = Following
editor-overlay-for-you = For You
editor-overlay-friends = Friends
editor-overlay-handle = @yourname
editor-overlay-handle-bare = yourname
editor-overlay-home = Home
editor-overlay-inbox = Inbox
editor-overlay-likes = 18K
editor-overlay-original-audio = yourname · Original audio
editor-overlay-original-sound = Original sound
editor-overlay-profile = Profile
editor-overlay-reels = Reels
editor-overlay-remix = Remix
editor-overlay-share = Share
editor-overlay-shorts = Shorts
editor-overlay-sound = original sound - yourname
editor-overlay-subscribe = Subscribe
editor-overlay-subscriptions = Subscriptions
editor-overlay-you = You

## The canvas sizes the Frame island offers, by platform and use, and the shapes a recording is made in.

editor-frame-size-youtube-1080 = YouTube 1080p
editor-frame-size-youtube-1440 = YouTube 1440p
editor-frame-size-youtube-2160 = YouTube 4K
editor-frame-size-youtube-shorts = YouTube Shorts
editor-frame-size-tiktok = TikTok
editor-frame-size-instagram-reels = Instagram Reels
editor-frame-size-instagram-portrait = Instagram portrait
editor-frame-size-instagram-square = Instagram square
editor-frame-size-linkedin-landscape = LinkedIn landscape
editor-frame-size-linkedin-square = LinkedIn square
editor-frame-size-linkedin-portrait = LinkedIn portrait
editor-frame-size-linkedin-vertical = LinkedIn vertical
editor-frame-size-x-landscape = X landscape
editor-frame-size-x-vertical = X vertical
editor-frame-size-youtube-thumbnail = YouTube thumbnail
editor-frame-size-youtube-thumbnail-4k = YouTube thumbnail 4K
# A photo post on TikTok, one picture or a slideshow, shown full screen.
editor-frame-size-tiktok-photo = TikTok photo
# Instagram's tallest portrait photo, 3:4, as against its 4:5 portrait.
editor-frame-size-instagram-tall-portrait = Instagram portrait 3:4
editor-frame-size-instagram-story = Instagram story
# A wide picture in a LinkedIn post, 1.91:1.
editor-frame-size-linkedin-image-landscape = LinkedIn landscape
editor-frame-size-x-image-landscape = X landscape
editor-frame-size-x-image-portrait = X portrait
editor-frame-size-facebook-post = Facebook post
editor-frame-size-pinterest-pin = Pinterest pin
# The picture a web page shows when its link is shared, on any network or chat app.
editor-frame-size-link-preview = Link preview
editor-frame-formats = Formats
# Names the Frame island's list of the platform templates the main canvas can be made for, which size it and keep it clear of that app's interface.
editor-frame-made-for = Format
# The Frame island's line that opens that list, named for assistive technology. $format is what the main canvas is made for, such as TikTok, or Custom.
editor-frame-made-for-current = Format: { $format }
# The Frame island's choice for a main canvas of a size of its own, made for no platform.
editor-frame-made-for-custom = Custom
# The format bar's button for the main canvas, the one worked in and sized freely, which every platform follows. Its visible label is its size, or the platform it is made for.
editor-frame-format-main = Main canvas
# The group of segments in the format bar for the platforms added, such as YouTube and TikTok.
editor-frame-format-platforms = Platforms
editor-frame-format-add = Add a platform
# A format named by its size, in pixels.
editor-frame-format-size = { $width } × { $height }
# A shape named by its proportions, width to height, such as 16:9 or 9:16.
editor-frame-format-ratio = { $width }:{ $height }
editor-frame-platform-youtube = YouTube
editor-frame-platform-tiktok = TikTok
editor-frame-platform-instagram = Instagram
editor-frame-platform-linkedin = LinkedIn
editor-frame-platform-x = X
editor-frame-platform-facebook = Facebook
editor-frame-platform-pinterest = Pinterest
# Heads templates for the web at large rather than one platform, such as a link preview.
editor-frame-platform-web = Web
# Removes one of the recording's formats, named.
editor-frame-format-remove-named = Remove { $format }
# The format bar's name for a screenshot's format that leaves out a layer or an annotation the other formats show, which a dot on its glyph marks. $format names the format.
editor-frame-format-leaves-out = { $format }, leaves something out
editor-frame-phone = Phone
editor-frame-phone-none = No phone
# Opens the choice of phone a tall format is previewed on. $phone names the one chosen, or that there is none.
editor-frame-phone-current = Phone: { $phone }
# Turns the phone and the app's interface over the picture on and off. They are only ever shown while editing.
editor-frame-phone-preview = Phone preview
editor-frame-phone-iphone-17-pro = iPhone 17 Pro
editor-frame-phone-iphone-17-pro-max = iPhone 17 Pro Max
editor-frame-phone-iphone-se = iPhone SE
editor-frame-phone-pixel-10 = Pixel 10
editor-frame-phone-galaxy-s26 = Galaxy S26
editor-frame-phone-galaxy-fold-cover = Galaxy Z Fold cover
# A toggle in the right-click menu of a scene, annotation or the captions: while on, it is laid out its own way where it is being edited, and changes to it there stay there. $shape names where: for a scene the platforms sharing its framing, such as "TikTok and YouTube Shorts"; for an annotation or the captions the proportion, such as 9:16.
editor-frame-format-custom = Custom for { $shape }
# In the right-click menu of something custom in the shape being edited: makes its layout there the one every shape uses.
editor-frame-format-layout-all = Use this layout in all shapes
# Heads the right-click menu's list of formats, each ticked where the item shows; pressing one shows it there or leaves it out.
editor-frame-format-show-in = Show in
