// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useRef, useState } from "react";

import { getScrubStrip } from "./api";

import type { ScrubStrip } from "./types";

/**
 * Each recording's scrub strip, by manifest, asked for the first time its
 * card is pointed at: null for one the editor has not composed yet. A strip
 * the editor composes again is forgotten, to be asked for afresh.
 */
export function useScrubStrips() {
  const [strips, setStrips] = useState<Record<string, ScrubStrip | null>>({});
  const requestedRef = useRef(new Set<string>());
  return {
    forget: (file: string) => {
      requestedRef.current.delete(file);
      setStrips((current) => {
        const { [file]: _forgotten, ...rest } = current;
        return rest;
      });
    },
    request: (file: string) => {
      if (requestedRef.current.has(file)) return;
      requestedRef.current.add(file);
      getScrubStrip(file)
        .then((strip) => {
          setStrips((current) => ({
            ...current,
            [file]: strip && {
              frameHeight: strip.frameHeight,
              frameWidth: strip.frameWidth,
              frames: strip.frames,
              // Its own address each time, as the editor rewrites it in place.
              src: `${convertFileSrc(strip.path)}?v=${String(Date.now())}`,
            },
          }));
        })
        .catch((cause: unknown) => {
          requestedRef.current.delete(file);
          console.error("Could not read the scrub strip", cause);
        });
    },
    strips,
  };
}
