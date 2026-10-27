// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { Text } from "../../text/text";

// What is in hand, said at the start of the island's row: a glyph in a
// circle's room where it has one, its name, and a line under it of when or
// how long. Without a glyph it keeps a little room instead, so the name
// clears the row's rounded end.

export function IslandRowTitle({
  glyph,
  note,
  title,
}: {
  /** Words, or words with parts such as keycaps set in them. */
  title: ReactNode;
  glyph?: ReactNode;
  note?: string;
}) {
  return (
    <span className="flex min-w-0 items-center gap-1 pe-2">
      {glyph ? (
        <span className="flex size-island-control shrink-0 items-center justify-center text-fg-secondary [&_svg]:size-icon">
          {glyph}
        </span>
      ) : (
        <span aria-hidden className="w-2" />
      )}
      <span className="flex min-w-0 flex-col">
        <Text as="span" className="max-w-72 truncate" variant="label">
          {title}
        </Text>
        {note ? (
          <Text
            as="span"
            className="max-w-72 truncate text-fg-secondary"
            variant="caption"
          >
            {note}
          </Text>
        ) : null}
      </span>
    </span>
  );
}
