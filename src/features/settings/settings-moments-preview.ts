// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { MomentSettings } from "./types";

/** The Moments pane's side of the Settings stories' API: kinds kept in
 * memory, microphone access granted, and two microphones to choose from.
 * Nothing reaches the app. */
export function momentsPreviewApi() {
  let moments: MomentSettings = {
    kinds: [
      {
        color: "#ffcc00",
        customColor: null,
        id: "funny",
        name: "Funny",
        shortcut: "CommandOrControl+Shift+Digit1",
      },
      {
        color: "#0088ff",
        // Chosen once, then left for a palette colour: the square keeps it.
        customColor: "#2ec4b6",
        id: "notable",
        name: "Notable",
        shortcut: "CommandOrControl+Shift+Digit2",
      },
    ],
    noteMicrophone: null,
    voiceNotes: true,
  };
  return {
    getMicrophoneAccess: () =>
      Promise.resolve({ canRequest: false, granted: true }),
    getMomentSettings: () => Promise.resolve(moments),
    listMicrophones: () =>
      Promise.resolve([
        { id: "built-in", isDefault: true, label: "MacBook Pro Microphone" },
        { id: "usb", label: "USB Microphone" },
      ]),
    openMicrophoneSettings: () => Promise.resolve(undefined),
    requestMicrophoneAccess: () => Promise.resolve(undefined),
    setMomentSettings: (next: MomentSettings) => {
      moments = next;
      return Promise.resolve(next);
    },
  };
}
