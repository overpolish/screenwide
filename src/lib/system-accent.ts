// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type SystemAccent = { blue: number; green: number; red: number };

// Removing the property lets the brand fallback in `--color-primary` apply.
const applySystemAccent = (accent: SystemAccent | null) => {
  const { style } = document.documentElement;
  if (accent) {
    const channels = [accent.red, accent.green, accent.blue].join(" ");
    style.setProperty("--system-accent", `rgb(${channels})`);
  } else {
    style.removeProperty("--system-accent");
  }
};

/**
 * Mirrors the OS accent colour into the `--system-accent` custom property, so
 * the `primary` tokens follow it. Components never read the accent themselves.
 */
export const synchronizeSystemAccent = () => {
  let unlisten: (() => void) | undefined;
  let stopped = false;

  // Outside the desktop app the command is absent or mocked; an unreadable
  // accent simply leaves the property unset.
  void invoke<SystemAccent | null>("get_system_accent")
    .then(applySystemAccent)
    .catch(() => {});

  void listen<SystemAccent | null>("system-accent-changed", ({ payload }) => {
    applySystemAccent(payload);
  })
    .then((stop) => {
      if (stopped) stop();
      else unlisten = stop;
    })
    .catch(() => {});

  return () => {
    stopped = true;
    unlisten?.();
  };
};
