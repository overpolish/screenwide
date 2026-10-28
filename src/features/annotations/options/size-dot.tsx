// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A size as a dot in the text's colour, as the tool's icon is: an arrow's
 * stroke or a counter's disc, each tool's sizes drawn in proportion. */
export function SizeDot({ diameter }: { diameter: number }) {
  return (
    <span
      className="rounded-full bg-current"
      style={{ height: diameter, width: diameter }}
    />
  );
}
