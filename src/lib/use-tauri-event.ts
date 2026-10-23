// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useLayoutEffect, useRef } from "react";

/** Stops a listener, which Tauri refuses with a rejection when asked before
 * the script that registers it in the page has run: in development React
 * mounts every component twice over, and the first can be gone that soon. */
const stop = (unlisten: UnlistenFn) => {
  void Promise.resolve()
    .then(unlisten)
    .catch(() => undefined);
};

/** Calls `handler` with each `event`'s payload while mounted. The latest
 * handler is the one called, so it may close over the latest render. */
export function useTauriEvent(
  event: string,
  handler: (payload: unknown) => void,
) {
  const handlerRef = useRef(handler);
  useLayoutEffect(() => {
    handlerRef.current = handler;
  });
  useEffect(() => {
    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen(event, ({ payload }) => {
      handlerRef.current(payload);
    }).then((started) => {
      if (disposed) stop(started);
      else unlisten = started;
    });
    return () => {
      disposed = true;
      if (unlisten) stop(unlisten);
    };
  }, [event]);
}
