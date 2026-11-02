// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { SAMPLE } from "./scenes/sample-palette";

import type { SystemCursor } from "../../../bindings/SystemCursor";

// The pointer a preview draws over its picture: the system's own arrow,
// read once from the app, or where it cannot be read, as in Storybook, the
// classic arrow painted in its colours.

export type CursorPicture = Omit<SystemCursor, "path"> & { src: string };

/** The classic arrow, in a 16 by 20 box, its tip at (1, 1). */
const ARROW = "M1 1 L1 16 L5 12 L8 19 L11 18 L8 11 L14 11 Z";

function paintedArrow(): CursorPicture {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 20"><path d="${ARROW}" fill="${SAMPLE.pointer}" stroke="${SAMPLE.pointerEdge}" stroke-width="1.2" stroke-linejoin="round"/></svg>`;
  return {
    height: 20,
    hotspotX: 1,
    hotspotY: 1,
    src: `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`,
    width: 16,
  };
}

let loaded: Promise<CursorPicture | null> | null = null;

export function useSystemCursor() {
  const [cursor, setCursor] = useState<CursorPicture>(paintedArrow);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    loaded ??= invoke<SystemCursor | null>("render_system_cursor").then(
      (found) => found && { ...found, src: convertFileSrc(found.path) },
    );
    loaded
      .then((found) => {
        if (current && found) setCursor(found);
      })
      .catch((error: unknown) => {
        console.error("The system's pointer could not be read", error);
      });
    return () => {
      current = false;
    };
  }, []);

  return cursor;
}
