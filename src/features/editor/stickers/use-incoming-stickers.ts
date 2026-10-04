// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";

import {
  readClipboardSticker,
  STICKER_DROP_EVENT,
  StickerDrop,
  StickerPoint,
} from "./sticker-api";

import type { StickerArt } from "../annotations/annotations";
import type { EditorKind } from "../types";

/**
 * Pictures arriving on the `workspace` editor as stickers: one pasted with
 * ⌘V or Ctrl+V outside a text field, placed in the middle of the layer in
 * hand, and one dropped on the picture, placed where it landed. Each is
 * placed by `place`, through the same commit a press with the sticker tool
 * makes, and kept in hand so it can be dressed at once; `onPlaced` then puts
 * a tool in hand that shows it. A clipboard with no picture leaves
 * everything as it was. `settle` runs first and is waited for, so a playing
 * recording has stopped before a sticker is placed on its still frame.
 */
export function useIncomingStickers(
  workspace: EditorKind,
  place:
    | ((art: StickerArt, at: StickerPoint | null) => Promise<boolean>)
    | undefined,
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
              const art = await readClipboardSticker();
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
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<StickerDrop>(STICKER_DROP_EVENT, ({ payload }) => {
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
