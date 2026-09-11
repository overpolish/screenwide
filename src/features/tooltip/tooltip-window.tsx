// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import {
  fitTooltip,
  getTooltip,
  listenToTooltipText,
  NativeTooltipContent,
} from "../../components/shared/native-tooltip/api";
import { NativeTooltipSurface } from "../../components/shared/native-tooltip/native-tooltip-surface";

/** What the window shows before its first tooltip. It is concealed until one
 * arrives, so this is never seen. */
const NOTHING: NativeTooltipContent = { label: "" };

/**
 * The Editor's tooltips as their own window.
 *
 * One window serves every control, so it stays loaded between tooltips: each
 * new one arrives as an event, and the window is never unmounted - its size is
 * measured from this surface, and a remount would leave that measurement
 * watching a discarded element.
 */
export function TooltipWindow() {
  const [content, setContent] = useState<NativeTooltipContent | null>(null);
  const surfaceRef = useRef<HTMLDivElement>(null);

  // The window is sized to the words and revealed only once it is: measured
  // after every tooltip has laid out, and again whenever the content moves, so
  // the tooltip never appears at one size and settles at another.
  useEffect(() => {
    const element = surfaceRef.current;
    if (!content || !element || !isTauri()) return;
    let frame = 0;
    const fit = () => {
      frame = 0;
      const rect = element.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;
      fitTooltip(rect.width, rect.height).catch((cause: unknown) => {
        console.error("Could not fit the tooltip", cause);
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
  }, [content]);

  useEffect(() => {
    let unmounted = false;
    let stopListening: (() => void) | undefined;
    // This mount is the first tooltip; the event carries every later one,
    // which this window stays loaded through.
    getTooltip()
      .then((current) => {
        if (!unmounted && current) setContent(current);
      })
      .catch((cause: unknown) => {
        console.error("Could not read the tooltip", cause);
      });
    listenToTooltipText((text) => {
      setContent(text);
    })
      .then((unlisten) => {
        if (unmounted) unlisten();
        else stopListening = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the tooltip", cause);
      });
    return () => {
      unmounted = true;
      stopListening?.();
    };
  }, []);

  return <NativeTooltipSurface content={content ?? NOTHING} ref={surfaceRef} />;
}
