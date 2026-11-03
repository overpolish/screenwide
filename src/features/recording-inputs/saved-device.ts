// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { InputDevice } from "./types";

/** The listed device a saved choice refers to. A device's id can change
 * while it stays the same device: on Windows a USB camera or microphone
 * plugged into another port comes back under a new one. It is then found
 * by its name, only when no other listed device shares that name, so two
 * alike are never swapped for each other. */
export function findSavedDevice<T extends InputDevice>(
  saved: InputDevice,
  devices: T[],
): T | null {
  const sameId = devices.find((device) => device.id === saved.id);
  if (sameId) return sameId;
  const sameName = devices.filter((device) => device.label === saved.label);
  return sameName.length === 1 ? sameName[0] : null;
}
