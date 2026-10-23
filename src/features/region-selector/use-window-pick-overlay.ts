// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../lib/use-tauri-event";
import {
  listMonitors,
  pickWindow,
  stepWindowPick,
} from "../recording-sources/api";
import {
  type MonitorDetails,
  type WindowPickHover,
} from "../recording-sources/types";

/** How long the overlay stays once picking ends, for the native cutout to fade
 * out (120 ms) where it was. */
const LEAVE_MS = 150;

type Pick = {
  /** The window under the pointer, or null over none. */
  hovered: WindowPickHover | null;
  /** Picking has ended, and the cutout is fading out. */
  isLeaving: boolean;
  /** Where the cutout was last, kept so it fades out there. */
  outline: WindowPickHover | null;
};

/** Keys that move the pick through the stacked windows: forwards, or back
 * with Shift. */
const STEPS: Record<string, number> = {
  ArrowDown: 1,
  ArrowLeft: -1,
  ArrowRight: 1,
  ArrowUp: -1,
  Tab: 1,
};

/** Picks the window the pick is on, for Return; over none, picking goes on.
 * A press never reaches the page: Rust catches it on the overlay and picks. */
const choose = () => {
  void pickWindow().catch((error: unknown) => {
    console.error("Could not pick the window", error);
  });
};

/**
 * Picking a window by clicking it, on the region overlay: while picking, the
 * window under the pointer, outlined as a region is but with nothing to
 * edit, on the display it is on. The overlay takes the keyboard, so Tab and
 * the arrows move the pick and Return makes it; a press on a window picks it
 * in Rust. Escape is the recording controls', which end the pick.
 */
export function useWindowPickOverlay() {
  const [pick, setPick] = useState<Pick | null>(null);
  const [monitors, setMonitors] = useState<MonitorDetails[]>([]);
  const leaveRef = useRef(0);
  useTauriEvent("window-pick://hovered", (payload) => {
    window.clearTimeout(leaveRef.current);
    const hovered = payload as WindowPickHover | null;
    setPick((current) => ({
      hovered,
      isLeaving: false,
      outline: hovered ?? current?.outline ?? null,
    }));
  });
  useTauriEvent("window-pick://ended", () => {
    setPick(
      (current) => current && { ...current, hovered: null, isLeaving: true },
    );
    window.clearTimeout(leaveRef.current);
    leaveRef.current = window.setTimeout(() => {
      setPick(null);
    }, LEAVE_MS);
  });
  useEffect(
    () => () => {
      window.clearTimeout(leaveRef.current);
    },
    [],
  );
  const isPicking = pick !== null;
  const isLeaving = pick?.isLeaving ?? false;
  const isActive = isPicking && !isLeaving;
  const hovered = pick?.hovered ?? null;
  const outline = pick?.outline ?? null;

  useEffect(() => {
    if (!isActive) return;
    let disposed = false;
    void listMonitors().then((next) => {
      if (!disposed) setMonitors(next);
    });
    void getCurrentWindow().setFocus();
    return () => {
      disposed = true;
    };
  }, [isActive]);

  useEffect(() => {
    if (!isActive) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Enter") {
        event.preventDefault();
        choose();
        return;
      }
      const step = STEPS[event.key] as number | undefined;
      if (step === undefined) return;
      event.preventDefault();
      void stepWindowPick(event.shiftKey ? -step : step);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [isActive]);

  const monitor = outline
    ? (monitors.find((candidate) => candidate.id === outline.monitorId) ?? null)
    : null;
  return { hovered, isLeaving, isPicking, monitor, outline };
}
