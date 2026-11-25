// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

import { frameSizePreset } from "../../tools/frame/frame-sizes";

import type { PlatformOverlay } from "../../../../bindings/PlatformOverlay";

/** The app whose interface a format made from `preset` is previewed under. */
export const appOf = (preset: string | null | undefined) =>
  frameSizePreset(preset)?.overlay ?? null;

/** Lays the app `overlay`, and the phone `phone` where one is chosen, over
 * the window's picture, a `width` by `height` canvas; nothing while
 * `isEnabled` is false, so only the editor showing formats lays one. */
export function usePlatformOverlay({
  height,
  isEnabled,
  overlay,
  phone,
  width,
}: {
  height: number;
  isEnabled: boolean;
  overlay: PlatformOverlay | null;
  phone: string | null;
  width: number;
}) {
  useEffect(() => {
    if (!isEnabled) return;
    // The canvas goes with it: the native side draws the overlay only once
    // the canvas on screen has this shape, which a format switch reaches a
    // moment after this does.
    invoke("set_editor_platform_overlay", {
      canvas: { height, width },
      phone: phone ? { phone } : null,
      platform: overlay,
    }).catch((cause: unknown) => {
      console.error(
        "Could not lay the platform overlay over the picture",
        cause,
      );
    });
  }, [height, isEnabled, overlay, phone, width]);
}
