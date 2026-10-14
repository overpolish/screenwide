// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The layer ids the native preview gives the overlays drawn over the
 * picture; the twins of `KEYBOARD_LAYER_ID` and `CAPTIONS_LAYER_ID` in
 * `preview_platform.rs`. */
export const KEYBOARD_LAYER_ID = 0xfffffffe;
export const CAPTIONS_LAYER_ID = 0xfffffffd;

export type OverlayLayerId =
  typeof KEYBOARD_LAYER_ID | typeof CAPTIONS_LAYER_ID;

export const isOverlayLayer = (layerId: number): layerId is OverlayLayerId =>
  layerId === KEYBOARD_LAYER_ID || layerId === CAPTIONS_LAYER_ID;

/**
 * What a press with the select tool can land on over the picture: the
 * captions, and the shortcuts where they are drawn. The native preview tests
 * a press against where each one shows at that moment, so the rectangle here
 * is only the canvas it is drawn on. A press on one chooses what it shows
 * then and opens its tool; it is never moved or sized there.
 */
export const overlayTargets = ({ keyboard }: { keyboard: boolean }) => [
  {
    layerId: CAPTIONS_LAYER_ID,
    paneIndex: 0,
    radiusPercent: 0,
    rect: { height: 1, width: 1, x: 0, y: 0 },
  },
  ...(keyboard
    ? [
        {
          layerId: KEYBOARD_LAYER_ID,
          paneIndex: 0,
          radiusPercent: 0,
          rect: { height: 1, width: 1, x: 0, y: 0 },
        },
      ]
    : []),
];
