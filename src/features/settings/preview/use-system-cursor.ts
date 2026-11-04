// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { SAMPLE } from "./scenes/sample-palette";

import type { SystemCursor } from "../../../bindings/SystemCursor";
import type { SystemCursorKind } from "../../../bindings/SystemCursorKind";

// The pointer a preview draws over its picture: the system's own arrow, or
// the crosshair the tools that measure or select the screen show, read once
// from the app, or where it cannot be read, as in Storybook, painted in the
// classic shape and colours.

export type CursorPicture = Omit<SystemCursor, "path"> & { src: string };

/** The classic arrow, in a 16 by 20 box, its tip at (1, 1). */
const ARROW = "M1 1 L1 16 L5 12 L8 19 L11 18 L8 11 L14 11 Z";
/** A crosshair's two hairlines, in a 17 by 17 box, crossing at (8.5, 8.5). */
const CROSSHAIR = "M8.5 0 V17 M0 8.5 H17";

function painted(kind: SystemCursorKind): CursorPicture {
  const [svg, width, height, hotspot] =
    kind === "crosshair"
      ? [
          `<path d="${CROSSHAIR}" stroke="${SAMPLE.pointerEdge}" stroke-width="3"/><path d="${CROSSHAIR}" stroke="${SAMPLE.pointer}" stroke-width="1"/>`,
          17,
          17,
          [8.5, 8.5],
        ]
      : [
          `<path d="${ARROW}" fill="${SAMPLE.pointer}" stroke="${SAMPLE.pointerEdge}" stroke-width="1.2" stroke-linejoin="round"/>`,
          16,
          20,
          [1, 1],
        ];
  const image = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${String(width)} ${String(height)}">${svg}</svg>`;
  return {
    height,
    hotspotX: hotspot[0],
    hotspotY: hotspot[1],
    src: `data:image/svg+xml;utf8,${encodeURIComponent(image)}`,
    width,
  };
}

const loaded = new Map<SystemCursorKind, Promise<CursorPicture | null>>();

function load(kind: SystemCursorKind) {
  let found = loaded.get(kind);
  if (!found) {
    found = invoke<SystemCursor | null>("render_system_cursor", { kind }).then(
      (cursor) => cursor && { ...cursor, src: convertFileSrc(cursor.path) },
    );
    loaded.set(kind, found);
  }
  return found;
}

export function useSystemCursor(kind: SystemCursorKind = "arrow") {
  const [cursor, setCursor] = useState(() => ({
    kind,
    picture: painted(kind),
  }));
  if (cursor.kind !== kind) {
    // Set while rendering, as React allows for state that follows a prop.
    setCursor({ kind, picture: painted(kind) });
  }

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    load(kind)
      .then((found) => {
        if (current && found) setCursor({ kind, picture: found });
      })
      .catch((error: unknown) => {
        console.error("The system's pointer could not be read", error);
      });
    return () => {
      current = false;
    };
  }, [kind]);

  return cursor.kind === kind ? cursor.picture : painted(kind);
}
