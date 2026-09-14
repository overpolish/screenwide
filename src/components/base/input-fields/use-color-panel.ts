// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

/** Tells the callers apart, so only the one that opened the panel hears it. */
let nextToken = 0;

/**
 * Whether the system Colours panel is available. WebKit's own colour popover
 * is a window we can neither place nor theme, so the Mac app asks AppKit for
 * the real panel instead; everywhere else keeps the native input.
 */
export const usesSystemColorPanel = () =>
  isTauri() && navigator.userAgent.includes("Mac");

/**
 * The system Colours panel, as one control owns it.
 *
 * AppKit's panel is a single shared window, so a caller takes it under a
 * token and hears only the events carrying that token back. Every drag step
 * arrives as a change; the close is the one moment the chosen colour is
 * settled, which is what anything keeping a colour should wait for rather
 * than writing every sample of a drag.
 */
export function useColorPanel({
  onChange,
  onClose,
}: {
  onChange: (value: string) => void;
  onClose?: (value: string | null) => void;
}) {
  const [token, setToken] = useState<string | null>(null);
  // The latest colour the panel reported, so a close knows what was settled
  // on without the caller having to hold it too.
  const latestRef = useRef<string | null>(null);
  const handlersRef = useRef({ onChange, onClose });
  handlersRef.current = { onChange, onClose };

  useEffect(() => {
    if (token === null) return;
    let listening = true;
    let unlisteners: UnlistenFn[] = [];
    const stop = () => {
      listening = false;
      for (const unlisten of unlisteners) unlisten();
      unlisteners = [];
    };
    void Promise.all([
      listen<{ color: string; token: string }>(
        "color-panel-change",
        (event) => {
          if (event.payload.token !== token) return;
          latestRef.current = event.payload.color;
          handlersRef.current.onChange(event.payload.color);
        },
      ),
      listen<{ token: string }>("color-panel-close", (event) => {
        if (event.payload.token !== token) return;
        setToken(null);
        handlersRef.current.onClose?.(latestRef.current);
        latestRef.current = null;
      }),
    ]).then((registered) => {
      if (!listening) {
        for (const unlisten of registered) unlisten();
        return;
      }
      unlisteners = registered;
    });
    return stop;
  }, [token]);

  return {
    isOpen: token !== null,
    /** Opens the panel showing `color`, and takes it for this caller. */
    open: (color: string) => {
      nextToken += 1;
      const requested = `color-well-${String(nextToken)}`;
      latestRef.current = null;
      setToken(requested);
      void invoke("show_color_panel", { color, token: requested });
    },
  };
}
