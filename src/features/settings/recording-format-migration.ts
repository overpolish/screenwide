// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getGeneralSettings, setGeneralSettings } from "./api";

import type { GeneralSettings } from "../../bindings/GeneralSettings";

// Earlier builds kept the frame rate and anti-flicker with the recording
// controls' inputs in the page's own storage, where only the window that
// wrote them could read them. They are settings now, stored with the rest;
// the inputs' storage hands over what it held the first time a newer build
// reads it, and keeps nothing of them after.

/** What `stored`, the inputs' storage from an earlier build, held of the
 * frame rate and anti-flicker, as the settings take them; nothing where it
 * chose neither. Anti-flicker was once chosen per camera, and was on if it
 * was on for any. */
export function legacyRecordingFormat(
  stored: unknown,
): Partial<Pick<GeneralSettings, "cameraAntiFlicker" | "recordingFps">> {
  if (!stored || typeof stored !== "object") return {};
  const { cameraPal, cameraPalById, fps } = stored as Record<string, unknown>;
  const perCamera =
    !!cameraPalById &&
    typeof cameraPalById === "object" &&
    Object.values(cameraPalById).some((pal) => pal === true);
  const antiFlicker =
    typeof cameraPal === "boolean" ? cameraPal : perCamera || undefined;
  return {
    ...(antiFlicker === undefined ? {} : { cameraAntiFlicker: antiFlicker }),
    ...(fps === 30 || fps === 60 ? { recordingFps: fps } : {}),
  };
}

/** Writes what the inputs' storage held into the settings. */
export async function moveRecordingFormat(
  changes: Partial<GeneralSettings>,
): Promise<void> {
  if (Object.keys(changes).length === 0) return;
  const settings = await getGeneralSettings();
  await setGeneralSettings({ ...settings, ...changes });
}
