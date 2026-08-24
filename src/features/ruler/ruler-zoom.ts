// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Convert a browser wheel delta into the same exponential zoom step as the
 * native export preview. Chromium reports a Windows wheel notch as roughly
 * 100 delta pixels; native receives that notch as 1 and applies `exp(0.12)`.
 * macOS sends fine-grained scrolling deltas and keeps its existing response.
 */
export function rulerWheelZoomFactor(deltaY: number, macOS: boolean) {
  return Math.exp(-deltaY * (macOS ? 0.01 : 0.0012));
}
