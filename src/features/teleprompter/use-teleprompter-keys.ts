// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef } from "react";

/** How far the arrow keys move the place, in words. */
const ARROW_WORDS = 8;

export type TeleprompterKeys = {
  isFullscreen: boolean;
  isPlaying: boolean;
  onClose: () => void;
  onFullscreenChange: (isFullscreen: boolean) => void;
  onPlace: (index: number) => void;
  onPlayingChange: (isPlaying: boolean) => void;
  /** Steps the text's size by `steps`. */
  onTextSize: (steps: number) => void;
  place: number;
};

/**
 * The teleprompter's keys. While the script is being written its field
 * takes the keys, but for Command or Control with + and −, which size the
 * text, and Escape, which stops writing. Otherwise Space plays and pauses,
 * the arrow keys move the place, Home goes back to the start, and Escape
 * pauses, then leaves the full display, then closes.
 */
export function useTeleprompterKeys(keys: TeleprompterKeys) {
  const keysRef = useRef(keys);
  keysRef.current = keys;
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const current = keysRef.current;
      const isCommand = event.metaKey || event.ctrlKey;
      const isWriting = event.target instanceof HTMLTextAreaElement;
      if (isCommand && (event.key === "=" || event.key === "+")) {
        current.onTextSize(1);
      } else if (isCommand && event.key === "-") {
        current.onTextSize(-1);
      } else if (event.key === "Escape" && isWriting) {
        event.target.blur();
      } else if (isWriting || isCommand || event.altKey) {
        return;
      } else if (event.key === " ") {
        // A focused button takes Space as a press of its own.
        if (event.target instanceof HTMLButtonElement) return;
        current.onPlayingChange(!current.isPlaying);
      } else if (event.key === "ArrowDown") {
        current.onPlace(Math.floor(current.place) + ARROW_WORDS);
      } else if (event.key === "ArrowUp") {
        current.onPlace(Math.floor(current.place) - ARROW_WORDS);
      } else if (event.key === "Home") {
        current.onPlace(0);
      } else if (event.key === "Escape") {
        if (current.isPlaying) current.onPlayingChange(false);
        else if (current.isFullscreen) current.onFullscreenChange(false);
        else current.onClose();
      } else {
        return;
      }
      event.preventDefault();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
}
