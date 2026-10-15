// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { CameraBackgroundRange } from "./camera-background-ranges";

/** What the camera panel's background rows do to the range they show. */
export type CameraBackgroundControls = {
  /** Show `background` behind the person over the range the panel shows,
   * adding one where there is none, or take the range away for `kept`. */
  setBackground: (
    background: CameraBackgroundRange["background"] | "kept",
  ) => void;
};

/**
 * The background range the camera panel shows, from the timeline: the one
 * chosen in its lane, or the one under the playhead. `hasRanges` says
 * whether the camera's background is taken away anywhere, which needs the
 * matte made.
 */
export type CameraBackgroundShown = {
  hasRanges: boolean;
  range: CameraBackgroundRange | null;
};

type Published = {
  controls: CameraBackgroundControls;
  shown: CameraBackgroundShown;
};

let published: Published | null = null;
const listeners = new Set<() => void>();

const notify = () => {
  for (const listener of listeners) listener();
};

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/** Publish the camera's background range for the panel for as long as the
 * timeline is mounted. `shown` is compared by value: the timeline rebuilds
 * it every render, and a panel re-rendered on identity alone would never
 * settle. */
export function usePublishCameraBackground(
  shown: CameraBackgroundShown,
  controls: CameraBackgroundControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  const serialized = JSON.stringify(shown);
  useEffect(() => {
    const next: Published = {
      // Read through the ref when called, so a panel holding this object
      // still reaches the timeline's current edit.
      controls: {
        setBackground: (background) => {
          controlsRef.current.setBackground(background);
        },
      },
      shown: JSON.parse(serialized) as CameraBackgroundShown,
    };
    published = next;
    notify();
    return () => {
      if (published !== next) return;
      published = null;
      notify();
    };
  }, [serialized]);
}

/** The camera's background range as the timeline publishes it, or null
 * while no recording timeline is mounted. */
export const usePublishedCameraBackground = () =>
  useSyncExternalStore(subscribe, () => published);
