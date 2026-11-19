// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import {
  CUE_BESIDE_EVENT,
  CUE_HEIGHT_EVENT,
  showTeleprompterCue,
} from "./cue-window-api";

import type { ShownTeleprompterCue } from "./cue-api";
import type { TeleprompterReadingLine } from "./cue-window-api";

/** The height the cue's window was last resized to by hand, in points. */
const HEIGHT_KEY = "screenwide.teleprompter.cue-height";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

const storedHeight = () => {
  const height = Number(window.localStorage.getItem(HEIGHT_KEY));
  return Number.isFinite(height) && height > 0 ? height : null;
};

/**
 * Shows the cue being read towards in its window beside the teleprompter,
 * level with the reading `line`, and puts it away as the reading passes it.
 * True while the cue stands there, or would; false where there is no room
 * beside the teleprompter, for it to show the cue over its text instead.
 */
export function useCueBeside(
  cue: ShownTeleprompterCue | null,
  isMirrored: boolean,
  line: TeleprompterReadingLine,
) {
  const [isBeside, setIsBeside] = useState(true);

  useEffect(() => {
    const unlistening = [
      listen<boolean>(CUE_BESIDE_EVENT, ({ payload }) => {
        setIsBeside(payload);
      }),
      listen<number>(CUE_HEIGHT_EVENT, ({ payload }) => {
        window.localStorage.setItem(HEIGHT_KEY, String(payload));
      }),
    ];
    return () => {
      for (const unlisten of unlistening)
        void unlisten.then((stop) => {
          stop();
        });
    };
  }, []);

  const path = cue?.path ?? null;
  const title = cue?.title ?? "";
  const { endIsLeft, frameTop, offset, share } = line;
  useEffect(() => {
    showTeleprompterCue({
      height: storedHeight(),
      line: { endIsLeft, frameTop, offset, share },
      picture: path === null ? null : { mirrored: isMirrored, path, title },
    })
      .then(setIsBeside)
      .catch(report("Could not show the cue"));
  }, [endIsLeft, frameTop, isMirrored, offset, path, share, title]);

  // The teleprompter going takes its cue with it.
  useEffect(
    () => () => {
      void showTeleprompterCue({ picture: null });
    },
    [],
  );

  return isBeside;
}
