// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CSSProperties } from "react";

// How tall an island may grow in its window, from its top down to the
// window's foot less the gap kept there, so a part of an island taller than
// the window (the Export island's body) scrolls inside it rather than being
// cut off. Whatever places an island (`PlacedIsland`, a story's stage)
// carries `data-island-room` and sets the variable on itself; what scrolls
// measures its own place from that element down.

/** Finds the element an island is placed in. */
export const ISLAND_ROOM_HOLDER = "[data-island-room]";

/** The custom property holding the room, a CSS length. */
export const ISLAND_ROOM = "--island-room";

/** The room as a style for the holder: `room` is a CSS length. */
export const islandRoomStyle = (room: string) =>
  ({ [ISLAND_ROOM]: room }) as CSSProperties;
