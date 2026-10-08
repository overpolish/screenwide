# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Text recognition, and the window describing a QR code found on screen.

## What a QR code holds, shown as the window title

text-recognition-qr-text = Text
text-recognition-qr-wifi = Wi-Fi network
text-recognition-qr-contact = Contact card
text-recognition-qr-calendar = Calendar event
text-recognition-qr-unsupported = Unsupported QR

## Actions a QR code offers, shown as the window title and its button

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

## The QR details window

text-recognition-qr-action-description = This QR code contains an action you can open or copy.
text-recognition-qr-unsupported-description = { -app-name } cannot perform this QR code’s action.
text-recognition-qr-information-description = This QR code contains information you can copy.
text-recognition-qr-content = Detected content
text-recognition-qr-copy-content = Copy detected content
text-recognition-qr-copy-failed = Could not copy QR content.
