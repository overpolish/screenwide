// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The page's size in points, width then height: the window a measured
 * viewport was laid out in. Sent with a layout, it lets the native side carry
 * the viewport by however far the window has been resized since. */
export const pageWindowSize = (): [number, number] => {
  // Fractional where the window's pixels do not divide into whole points,
  // as the viewport's own measure is.
  const viewport = window.visualViewport;
  return viewport
    ? [viewport.width, viewport.height]
    : [window.innerWidth, window.innerHeight];
};
