// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { Alert } from "./alert";
import {
  AlertCopy,
  dismissAlert,
  fitAlert,
  getAlert,
  listenToAlertShown,
} from "./api";

const dismiss = () => {
  dismissAlert().catch((cause: unknown) => {
    console.error("Could not dismiss the alert", cause);
  });
};

/**
 * The shared alert as its own window.
 *
 * One window serves every alert, so it stays loaded between them and each new
 * one arrives as an event. The alert is never unmounted: the window's height
 * is measured from it, and a remount would leave that measurement watching a
 * discarded element.
 */
export function AlertWindow() {
  const [copy, setCopy] = useState<AlertCopy | null>(null);
  const contentRef = useRef<HTMLElement>(null);

  // The window is sized from the words and revealed only once it is, so the
  // alert never appears at one height and settles at another.
  useEffect(() => {
    const element = contentRef.current;
    if (!copy || !element || !isTauri()) return;
    let frame = 0;
    const fit = () => {
      frame = 0;
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height === 0) return;
      fitAlert(height).catch((cause: unknown) => {
        console.error("Could not fit the alert", cause);
      });
    };
    const schedule = () => {
      frame ||= window.requestAnimationFrame(fit);
    };
    const observer = new ResizeObserver(schedule);
    observer.observe(element);
    schedule();
    return () => {
      observer.disconnect();
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [copy]);

  useEffect(() => {
    let unmounted = false;
    let stopListening: (() => void) | undefined;
    // This mount is the first alert; the event carries every later one.
    getAlert()
      .then((current) => {
        if (!unmounted && current) setCopy(current);
      })
      .catch((cause: unknown) => {
        console.error("Could not read the alert", cause);
      });
    listenToAlertShown(setCopy)
      .then((unlisten) => {
        if (unmounted) unlisten();
        else stopListening = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the alert", cause);
      });
    return () => {
      unmounted = true;
      stopListening?.();
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat) return;
      if (event.key === "Escape" || event.key === "Enter") dismiss();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    <Alert
      contentRef={contentRef}
      message={copy?.message ?? ""}
      onDismiss={dismiss}
      title={copy?.title ?? ""}
    />
  );
}
