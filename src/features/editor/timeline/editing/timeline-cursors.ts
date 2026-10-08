// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Exact Lucide geometry. Custom CSS cursors are rasterized by the WebView, so
// density variants keep OS-level cursor enlargement from starting at 24 px.
const SCISSORS_PATHS = `
  <circle cx="6" cy="6" r="3"/>
  <path d="M8.12 8.12 12 12"/>
  <path d="M20 4 8.12 15.88"/>
  <circle cx="6" cy="18" r="3"/>
  <path d="M14.8 14.8 20 20"/>
`;
const isMacOS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
// A cursor is rasterized from a data URI, which cannot reach the document's
// custom properties, so these literals mirror the theme tokens the cursor is
// drawn from: the glyph takes a label colour (`--color-content-fg`) and its
// outline the window colour behind it (`--color-content`). macOS draws its
// cursors dark on light, Windows the other way round, so each platform takes
// the pair from the appearance its cursors read against.
const CONTENT_FG_LIGHT = "rgba(0, 0, 0, 0.85)";
const CONTENT_LIGHT = "rgb(255, 255, 255)";
const CONTENT_FG_DARK = "rgba(255, 255, 255, 0.85)";
const CONTENT_DARK = "rgb(30, 30, 30)";
const cursorSvg = (paths: string, density: number) => {
  const iconColor = isMacOS ? CONTENT_FG_LIGHT : CONTENT_FG_DARK;
  const outlineColor = isMacOS ? CONTENT_LIGHT : CONTENT_DARK;
  const size = 24 * density;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size.toString()}" height="${size.toString()}" viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round"><g stroke="${outlineColor}" stroke-width="5">${paths}</g><g stroke="${iconColor}" stroke-width="2">${paths}</g></svg>`;
};
const cursorImageSet = (paths: string) =>
  `image-set(${[1, 2, 3]
    .map(
      (density) =>
        `url("data:image/svg+xml,${encodeURIComponent(cursorSvg(paths, density))}") ${density.toString()}x`,
    )
    .join(", ")})`;

export const TIMELINE_BLADE_CURSOR = `${cursorImageSet(SCISSORS_PATHS)} 12 12, crosshair`;

const arrowLeftToLinePaths =
  '<path d="M3 19V5"/><path d="m13 6-6 6 6 6"/><path d="M7 12h14"/>';
const arrowRightToLinePaths =
  '<path d="M17 12H3"/><path d="m11 18 6-6-6-6"/><path d="M21 5v14"/>';
export const TRIM_LEFT_CURSOR = `${cursorImageSet(arrowLeftToLinePaths)} 12 12, ew-resize`;
export const TRIM_RIGHT_CURSOR = `${cursorImageSet(arrowRightToLinePaths)} 12 12, ew-resize`;
