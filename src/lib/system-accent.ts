// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Rgb = [red: number, green: number, blue: number];
type SystemAccent = {
  blue: number;
  green: number;
  red: number;
};

declare global {
  interface Window {
    /** Seeded by Rust's `system_accent::initialization_script` before the
     * page runs, on windows built after launch; absent on the rest. */
    __SCREENWIDE_ACCENT__?: SystemAccent | null;
  }
}

const rgb = (channels: Rgb) => `rgb(${channels.join(" ")})`;

// Removing the property lets the brand orange in `--color-emphasis` apply.
const applySystemAccent = (accent: SystemAccent | null) => {
  const { style } = document.documentElement;
  if (accent) {
    style.setProperty(
      "--system-accent",
      rgb([accent.red, accent.green, accent.blue]),
    );
  } else {
    style.removeProperty("--system-accent");
  }
};

/**
 * Mirrors the OS accent colour into the `--system-accent` custom property, so
 * the emphasis (focus, the playhead, progress) follows it. Components never
 * read the accent themselves.
 */
export const synchronizeSystemAccent = () => {
  let unlisten: (() => void) | undefined;
  let stopped = false;

  // A window built after launch is shown while its page still loads, so Rust
  // seeds it with the accent through an initialization script
  // (`system_accent::initialization_script`); applied here, before the first
  // paint, it never flashes the brand colour. The windows from the config
  // load hidden and have none.
  const seeded = window.__SCREENWIDE_ACCENT__;
  if (seeded !== undefined) applySystemAccent(seeded);

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
