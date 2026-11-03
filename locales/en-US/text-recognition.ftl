# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Text recognition: its tools on the island, and what a QR code found on screen holds.

## The island while text recognition is up

text-recognition-island = Text recognition
text-recognition-island-choosing = Drag over text or a QR code
text-recognition-island-failed = Could not read that area. Try another.
text-recognition-island-reading = Finding text and QR codes
# Names the spinner beside the line above, for assistive technology.
text-recognition-island-reading-label = Reading the area
# Copies the selected text; shown while some is selected.
text-recognition-island-copy = Copy
# Copies everything found; shown while nothing is selected.
text-recognition-island-copy-all = Copy all
# Copies the selected text, or all of it, joined into one paragraph.
text-recognition-island-copy-paragraph = Copy as paragraph
# Shown when only QR codes were found.
text-recognition-island-qr-hint = Click a QR code to see what it holds
text-recognition-island-nothing-found = No text or QR codes found
text-recognition-island-select-another = Select another area
text-recognition-island-close = Close text recognition

## A QR code pressed in what was found, on the island

# Goes back from the code to what was found.
text-recognition-island-qr-back = Back to what was found
# Copies what the code holds.
text-recognition-island-qr-copy = Copy what the code holds
# Names what the code holds, under the island's row.
text-recognition-island-qr-content = What the code holds
# Shows the whole of what the code holds where it is cut to a few lines, and cuts it again.
text-recognition-island-qr-show-all = Show all
text-recognition-island-qr-show-less = Show less
text-recognition-qr-copy-failed = Could not copy QR content.

## What a QR code holds, shown as the island's title

text-recognition-qr-text = Text
text-recognition-qr-wifi = Wi-Fi network
text-recognition-qr-contact = Contact card
text-recognition-qr-calendar = Calendar event
text-recognition-qr-link = Link
text-recognition-qr-phone = Phone number
text-recognition-qr-email-address = Email address
text-recognition-qr-sms = Text message
text-recognition-qr-unsupported = Unsupported QR

## Actions a QR code offers, shown as the island's button

text-recognition-qr-open-link = Open link
text-recognition-qr-call = Call number
text-recognition-qr-email = Compose email
text-recognition-qr-message = Compose message
# $action is link, phone, email or sms.
text-recognition-qr-action-failed =
    { $action ->
        [phone] Could not call number.
        [email] Could not compose email.
        [sms] Could not compose message.
       *[link] Could not open link.
    }

## Why a QR code cannot be used

text-recognition-qr-no-content = QR code has no content.
text-recognition-qr-malformed = QR code contains a malformed action.
text-recognition-qr-wifi-incomplete = Wi-Fi QR is missing a network name.
text-recognition-qr-contact-incomplete = Contact QR is incomplete.
text-recognition-qr-calendar-incomplete = Calendar QR is incomplete.
text-recognition-qr-link-incomplete = Web QR is missing a destination.
text-recognition-qr-phone-incomplete = Phone QR is missing a number.
text-recognition-qr-email-incomplete = Email QR is missing a recipient.
text-recognition-qr-message-incomplete = Message QR is missing a recipient.
# $scheme is the action's URL scheme, such as "javascript".
text-recognition-qr-unsupported-action = The { $scheme } action is not supported.
