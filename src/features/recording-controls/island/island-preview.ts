// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  IslandDisplay,
  IslandInput,
  IslandRegion,
  IslandWindow,
} from "./island-types";

// Sample devices and displays for the island's stories. The input lists are
// long enough to scroll, and the cameras come in several shapes.

/** A stand-in display still: a wallpaper of two colours, at a display's
 * shape. The app falls back to a plain fill where a still has not come. */
const wallpaper = (from: string, to: string) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 100"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="160" height="100" fill="url(#g)"/></svg>`,
  )}`;

export const PREVIEW_DISPLAYS: IslandDisplay[] = [
  {
    id: 1,
    label: "Built-in Display",
    thumbnail: wallpaper("#5b7cfa", "#b06ab3"),
  },
  {
    id: 2,
    label: "Studio Display",
    thumbnail: wallpaper("#f6a65b", "#c4456b"),
  },
];

/** A MacBook's notch, as its screen reports it, for the stories. */
export const PREVIEW_NOTCH = { height: 32, width: 190 };

/** A stand-in app icon: a rounded square in two colours. */
const appIcon = (from: string, to: string) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect x="4" y="4" width="56" height="56" rx="14" fill="url(#g)"/></svg>`,
  )}`;

/** Windows to pick in the stories, in turn. */
export const PREVIEW_WINDOWS: IslandWindow[] = [
  {
    appName: "Safari",
    icon: appIcon("#5ac8fa", "#0a64d6"),
    title: "Screenwide: Release notes",
  },
  {
    appName: "Notes",
    icon: appIcon("#ffe066", "#f2b705"),
    title: "Launch checklist for the recording island redesign",
  },
];

/** Areas to drag out in the stories, in turn: one on a display, one across
 * two. */
export const PREVIEW_REGIONS: IslandRegion[] = [
  { displays: ["Built-in Display"], height: 720, width: 1280 },
  {
    displays: ["Built-in Display", "Studio Display"],
    height: 900,
    width: 3200,
  },
];

/** A desk of displays, more than fit one row. */
export const PREVIEW_MANY_DISPLAYS: IslandDisplay[] = [
  ...PREVIEW_DISPLAYS,
  {
    id: 3,
    label: "LG UltraFine 5K",
    thumbnail: wallpaper("#3fb7a5", "#1d5f8a"),
  },
  { id: 4, label: "Dell U2723QE", thumbnail: wallpaper("#8a8f98", "#2d3240") },
  {
    id: 5,
    label: "Sidecar iPad Pro",
    thumbnail: wallpaper("#ffd36b", "#e86a3c"),
  },
];

export const previewCamera = (
  input: Partial<IslandInput> = {},
): IslandInput => ({
  devices: [
    { id: "facetime", label: "FaceTime HD Camera" },
    { aspectRatio: 9 / 16, id: "continuity", label: "iPhone Camera" },
    { id: "studio", label: "Studio Display Camera" },
    { id: "brio", label: "Logitech BRIO" },
    { id: "facecam", label: "Elgato Facecam Pro" },
    { aspectRatio: 3 / 2, id: "zv-e10", label: "Sony ZV-E10" },
    { id: "link", label: "Insta360 Link 2" },
    { id: "kiyo", label: "Razer Kiyo Pro" },
    { aspectRatio: 4 / 3, id: "opal", label: "Opal C1" },
    { aspectRatio: 1, id: "obs", label: "OBS Virtual Camera" },
  ],
  isOn: true,
  selectedIds: ["facetime"],
  ...input,
});

export const previewMicrophone = (
  input: Partial<IslandInput> = {},
): IslandInput => ({
  devices: [
    { id: "builtin", label: "MacBook Pro Microphone" },
    { id: "shure", label: "Shure MV7+" },
    { id: "airpods", label: "AirPods Pro" },
    { id: "studio", label: "Studio Display Microphone" },
    { id: "iphone", label: "iPhone Microphone" },
    { id: "nt-usb", label: "RØDE NT-USB+" },
    { id: "wave", label: "Elgato Wave:3" },
    { id: "yeti", label: "Blue Yeti" },
    { id: "scarlett", label: "Scarlett 2i2 USB" },
    { id: "zoom-h6", label: "Zoom H6" },
    { id: "xm5", label: "Sony WH-1000XM5" },
    { id: "brio", label: "Logitech BRIO" },
  ],
  isOn: true,
  selectedIds: ["shure"],
  ...input,
});

export const previewSystemAudio = (
  input: Partial<IslandInput> = {},
): IslandInput => ({
  devices: [
    { id: "all", isPinned: true, label: "All audio" },
    { icon: appIcon("#5ac8fa", "#0a64d6"), id: "safari", label: "Safari" },
    { icon: appIcon("#ff7a8a", "#e0294a"), id: "music", label: "Music" },
    { icon: appIcon("#5ee08a", "#1aa34a"), id: "spotify", label: "Spotify" },
    { icon: appIcon("#7aa7ff", "#2d64f0"), id: "zoom", label: "zoom.us" },
    { icon: appIcon("#c58af9", "#6a2bb8"), id: "slack", label: "Slack" },
    {
      icon: appIcon("#ffd36b", "#e8a10c"),
      id: "chrome",
      label: "Google Chrome",
    },
    {
      icon: appIcon("#9a9aa3", "#3c3c44"),
      id: "final-cut",
      label: "Final Cut Pro",
    },
    { icon: appIcon("#8a8a92", "#2a2a30"), id: "logic", label: "Logic Pro" },
    {
      icon: appIcon("#a3d8ff", "#3e8ed8"),
      id: "quicktime",
      label: "QuickTime Player",
    },
    { icon: appIcon("#7fc3ff", "#1f6fd1"), id: "keynote", label: "Keynote" },
    { icon: appIcon("#9aa4ff", "#5865f2"), id: "discord", label: "Discord" },
  ],
  isOn: true,
  selectedIds: ["all"],
  ...input,
});
