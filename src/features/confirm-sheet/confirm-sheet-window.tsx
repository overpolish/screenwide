// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  ConfirmSheetCopy,
  fitConfirmSheet,
  getConfirmSheet,
  listenToConfirmSheetAsked,
  resolveConfirmSheet,
} from "./api";
import { ConfirmSheet } from "./confirm-sheet";

/** What the sheet shows before its first question. The window is hidden until
 * one arrives, so this is never seen. */
const NO_QUESTION: ConfirmSheetCopy = {
  cancelLabel: "Cancel",
  confirmLabel: "OK",
  message: "",
  title: "",
};

/**
 * The shared confirmation sheet as its own window.
 *
 * It owns nothing but the answer: the words come from whoever asked, and the
 * answer goes straight back to them. One window serves every question, so it
 * stays loaded between them, each new question arrives as an event, and the
 * sheet is never unmounted - the window's height is measured from it, and a
 * remount would leave that measurement watching a discarded element.
 */
export function ConfirmSheetWindow() {
  const [copy, setCopy] = useState<ConfirmSheetCopy | null>(null);
  const contentRef = useRef<HTMLElement>(null);

  // The window is sized from the words and shown only once it is: measured
  // after every question has laid out, and again whenever the content moves,
  // so the sheet never appears at one height and settles at another.
  useEffect(() => {
    const element = contentRef.current;
    if (!copy || !element || !isTauri()) return;
    let frame = 0;
    const fit = () => {
      frame = 0;
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height === 0) return;
      fitConfirmSheet(height).catch((cause: unknown) => {
        console.error("Could not fit the confirmation sheet", cause);
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
    // This mount is the first question; the event carries every later one,
    // which this window stays loaded through.
    getConfirmSheet()
      .then((current) => {
        if (!unmounted && current) setCopy(current);
      })
      .catch((cause: unknown) => {
        console.error("Could not read the confirmation", cause);
      });
    listenToConfirmSheetAsked((asked) => {
      setCopy(asked);
    })
      .then((unlisten) => {
        if (unmounted) unlisten();
        else stopListening = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the confirmation sheet", cause);
      });
    return () => {
      unmounted = true;
      stopListening?.();
    };
  }, []);

  const answer = useCallback((confirmed: boolean) => {
    resolveConfirmSheet(confirmed).catch((cause: unknown) => {
      console.error("Could not answer the confirmation", cause);
    });
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      // Escape is the way out of a sheet, and the way out is never the action
      // the sheet is asking about. Return takes the default action, so the
      // button need not hold focus and draw a ring on arrival.
      if (event.key === "Escape") answer(false);
      if (event.key === "Enter" && !event.repeat) answer(true);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [answer]);

  return (
    <ConfirmSheet
      {...(copy ?? NO_QUESTION)}
      contentRef={contentRef}
      onCancel={() => {
        answer(false);
      }}
      onConfirm={() => {
        answer(true);
      }}
    />
  );
}
