// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";

import {
  readClipboardImage,
  IMAGE_DROP_EVENT,
  ImageDrop,
  ImagePoint,
} from "./image-api";
import { registerImagePlacer } from "./image-requests";

import type { ImageArt } from "../annotations/annotations";
import type { EditorKind } from "../types";

/**
 * Pictures arriving on the `workspace` editor as images: one pasted with
 * ⌘V or Ctrl+V outside a text field, or chosen in the panel with no image
 * in hand, placed in the middle of the layer in hand, and one dropped on the
 * picture, placed where it landed. Each is placed by `place`, through the
 * same commit a press with the image tool makes, and kept in hand so it can
 * be dressed at once; `onPlaced` then puts a tool in hand that shows it. A
 * clipboard with no picture leaves everything as it was. `settle` runs first
 * and is waited for, so a playing recording has stopped before an image is
 * placed on its still frame.
 */
export function useIncomingImages(
  workspace: EditorKind,
  place:
    ((art: ImageArt, at: ImagePoint | null) => Promise<boolean>) | undefined,
  { onPlaced, settle }: { onPlaced: () => void; settle?: () => Promise<void> },
) {
  const placeRef = useRef(place);
  placeRef.current = place;
  const settleRef = useRef(settle);
  settleRef.current = settle;
  const onPlacedRef = useRef(onPlaced);
  onPlacedRef.current = onPlaced;
  // A held ⌘V repeats; one picture is read at a time.
  const readingRef = useRef(false);
  useEditorWindowShortcuts({
    onPaste: place
      ? () => {
          if (readingRef.current) return;
          readingRef.current = true;
          void (async () => {
            try {
              await settleRef.current?.();
              const art = await readClipboardImage();
              if (art && (await place(art, null))) onPlacedRef.current();
            } catch (cause) {
              console.error("Could not paste the picture", cause);
            } finally {
              readingRef.current = false;
            }
          })();
        }
      : undefined,
  });
  useEffect(() => {
    if (!place) return;
    return registerImagePlacer(workspace, (art) => {
      void (async () => {
        try {
          await settleRef.current?.();
          if (await place(art, null)) onPlacedRef.current();
        } catch (cause) {
          console.error("Could not place the chosen picture", cause);
        }
      })();
    });
  }, [place, workspace]);
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<ImageDrop>(IMAGE_DROP_EVENT, ({ payload }) => {
      const placing = placeRef.current;
      if (disposed || payload.workspace !== workspace || !placing) return;
      void (async () => {
        try {
          await settleRef.current?.();
          if (await placing(payload.art, payload.at)) onPlacedRef.current();
        } catch (cause) {
          console.error("Could not place the dropped picture", cause);
        }
      })();
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [workspace]);
}
