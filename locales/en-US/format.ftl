# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### Quantities written the same way across the app.

# A file size. $size is the number, already written in the reader's own
# number format; $unit is byte, kilobyte, megabyte, gigabyte or terabyte.
# Finder and Explorer write the units this way.
format-size =
    { $unit ->
        [kilobyte] { $size } KB
        [megabyte] { $size } MB
        [gigabyte] { $size } GB
        [terabyte] { $size } TB
       *[byte] { $size } bytes
    }
format-size-unknown = Unknown size
