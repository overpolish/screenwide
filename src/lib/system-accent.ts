// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Rgb = [red: number, green: number, blue: number];
type SystemAccent = {
  blue: number;
  green: number;
  red: number;
  /** The tones the platform's own controls fill with per appearance, and
   * the ones it sets accent text in, where those differ from the accent
   * (Windows). Absent on macOS. */
  tones?: { dark: Rgb; darkText: Rgb; light: Rgb; lightText: Rgb };
};

declare global {
  interface Window {
    /** Seeded by Rust's `system_accent::initialization_script` before the
     * page runs, on windows built after launch; absent on the rest. */
    __SCREENWIDE_ACCENT__?: SystemAccent | null;
  }
}

const rgb = (channels: Rgb) => `rgb(${channels.join(" ")})`;

// Removing the properties lets the brand fallback in `--color-primary` and
// the platform's fill tokens apply.
const applySystemAccent = (accent: SystemAccent | null) => {
  const { style } = document.documentElement;
  const tones = accent?.tones;
  if (accent) {
    style.setProperty(
      "--system-accent",
      rgb([accent.red, accent.green, accent.blue]),
    );
  } else {
    style.removeProperty("--system-accent");
  }
  if (tones) {
    style.setProperty("--system-accent-light-tone", rgb(tones.light));
    style.setProperty("--system-accent-dark-tone", rgb(tones.dark));
    style.setProperty("--system-accent-light-text", rgb(tones.lightText));
    style.setProperty("--system-accent-dark-text", rgb(tones.darkText));
    // WinUI sets black text on its dark-appearance accent tone; the brand
    // colour keeps white text, so this only exists while the OS tones do.
    style.setProperty("--system-accent-dark-tone-fg", "black");
  } else {
    style.removeProperty("--system-accent-light-tone");
    style.removeProperty("--system-accent-dark-tone");
    style.removeProperty("--system-accent-light-text");
    style.removeProperty("--system-accent-dark-text");
    style.removeProperty("--system-accent-dark-tone-fg");
  }
};

/**
 * Mirrors the OS accent colour into the `--system-accent` custom property, so
 * the `primary` tokens follow it. Components never read the accent themselves.
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
