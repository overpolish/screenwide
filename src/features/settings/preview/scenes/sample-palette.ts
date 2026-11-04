// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The colours of the sample desktop the previews show a setting at work on.
// Its windows are other people's apps, not Screenwide, so they do not wear
// Screenwide's tokens: an app in light mode looks the same whatever
// appearance Settings is in, and its accent is its own. The platform's
// window buttons are the system's colours. Everything Screenwide itself
// draws on the desktop, its island and its tools, keeps the app's tokens.

export const SAMPLE = {
  /** The apps' own accent: their primary button, their chart. */
  accent: "#2f6fed",
  /** Windows' caption glyphs. */
  caption: "#1d1d1f",
  caret: "#2f6fed",
  chart: ["#2f6fed", "#8fb2ff", "#f2994a", "#36b37e"],
  /** The desktop's icon labels, and the shadow under them and the icons. */
  desktopInk: "#ffffff",
  desktopShadow: "rgb(0 0 0 / 45%)",
  field: "#ffffff",
  fieldEdge: "#cfccc6",
  /** A folder on the desktop, and its tab. */
  folder: "#6cb6f5",
  folderTab: "#4f9ee6",
  hairline: "rgb(0 0 0 / 10%)",
  heading: "#1d1d1f",
  /** Stand-in app icons on the taskbar. */
  icons: ["#2f6fed", "#f2994a", "#36b37e", "#9b6bdf"],
  /** Lines of text too small to read, as bars. */
  line: "#dcd9d4",
  lineStrong: "#c9c5bf",
  /** The macOS menu bar and the Windows taskbar, translucent over the
   * desktop picture as the system draws them. */
  menuBar: "rgb(255 255 255 / 55%)",
  menuBarInk: "rgb(0 0 0 / 55%)",
  /** The display's notch, the black of the bezel. */
  notch: "#000000",
  onAccent: "#ffffff",
  /** The system's pointer, where the app cannot read the real one. */
  pointer: "#000000",
  pointerEdge: "#ffffff",
  /** The window's shadow, dark and wide as a focused window's is. */
  shadow: "rgb(0 0 0 / 32%)",
  sidebar: "#f1efec",
  sidebarSelected: "#e2dfda",
  taskbar: "rgb(243 243 243 / 82%)",
  text: "#3c3c40",
  titleBar: "#f6f5f3",
  /** macOS's window buttons: close, minimise, zoom. */
  trafficLights: ["#ff5f57", "#febc2e", "#28c840"],
  /** Behind everything where the system names no desktop picture: a
   * dusk-coloured mesh, from the deepest colour out. */
  wallpaper: ["#1d2a4d", "#4b3f8f", "#c06c8e", "#f0a868", "#2f6d8f"],
  window: "#ffffff",
} as const;
