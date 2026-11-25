// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { PlatformOverlay } from "../../../../bindings/PlatformOverlay";

/** A canvas size a platform asks for, named for the platform and use. */
export type FrameSizePreset = {
  height: number;
  id: FrameSizePresetId;
  width: number;
  /** The app whose interface a video this size plays under. */
  overlay?: PlatformOverlay;
};

export type FrameSizePresetId =
  | "youtube-1080"
  | "youtube-1440"
  | "youtube-2160"
  | "youtube-shorts"
  | "tiktok"
  | "instagram-reels"
  | "instagram-portrait"
  | "instagram-square"
  | "linkedin-landscape"
  | "linkedin-square"
  | "linkedin-portrait"
  | "linkedin-vertical"
  | "x-landscape"
  | "x-vertical"
  | ScreenshotSizePresetId;

/** The templates a screenshot is made for, which are pictures rather than
 * videos: a thumbnail, a photo post, a story, a pin, a link preview. */
type ScreenshotSizePresetId =
  | "youtube-thumbnail"
  | "youtube-thumbnail-4k"
  | "tiktok-photo"
  | "instagram-tall-portrait"
  | "instagram-story"
  | "linkedin-image-landscape"
  | "x-image-landscape"
  | "x-image-portrait"
  | "facebook-post"
  | "pinterest-pin"
  | "link-preview";

const LANDSCAPE = { height: 1080, width: 1920 } as const;
const VERTICAL = { height: 1920, width: 1080 } as const;
const PORTRAIT = { height: 1350, width: 1080 } as const;
const TALL_PORTRAIT = { height: 1440, width: 1080 } as const;
const SQUARE = { height: 1080, width: 1080 } as const;

/** The sizes each platform publishes for its feeds, platform by platform. */
export const FRAME_SIZE_PRESETS: readonly FrameSizePreset[] = [
  { id: "youtube-1080", ...LANDSCAPE },
  { height: 1440, id: "youtube-1440", width: 2560 },
  { height: 2160, id: "youtube-2160", width: 3840 },
  { id: "youtube-shorts", overlay: "youtube-shorts", ...VERTICAL },
  { id: "tiktok", overlay: "tiktok", ...VERTICAL },
  { id: "instagram-reels", overlay: "instagram-reels", ...VERTICAL },
  { id: "instagram-portrait", ...PORTRAIT },
  { id: "instagram-square", ...SQUARE },
  { id: "linkedin-landscape", ...LANDSCAPE },
  { id: "linkedin-square", ...SQUARE },
  { id: "linkedin-portrait", ...PORTRAIT },
  { id: "linkedin-vertical", ...VERTICAL },
  { id: "x-landscape", ...LANDSCAPE },
  { id: "x-vertical", ...VERTICAL },
];

/** The sizes each platform publishes pictures at, platform by platform, as
 * their own help pages give them. A TikTok photo post is shown full screen
 * under the app, as its videos are. */
export const SCREENSHOT_SIZE_PRESETS: readonly FrameSizePreset[] = [
  { height: 720, id: "youtube-thumbnail", width: 1280 },
  { height: 2160, id: "youtube-thumbnail-4k", width: 3840 },
  { id: "tiktok-photo", overlay: "tiktok", ...VERTICAL },
  { id: "instagram-portrait", ...PORTRAIT },
  { id: "instagram-tall-portrait", ...TALL_PORTRAIT },
  { id: "instagram-square", ...SQUARE },
  { id: "instagram-story", ...VERTICAL },
  { id: "linkedin-portrait", ...PORTRAIT },
  { id: "linkedin-square", ...SQUARE },
  { height: 627, id: "linkedin-image-landscape", width: 1200 },
  { height: 900, id: "x-image-landscape", width: 1600 },
  { id: "x-image-portrait", ...TALL_PORTRAIT },
  { id: "facebook-post", ...PORTRAIT },
  { height: 1500, id: "pinterest-pin", width: 1000 },
  { height: 630, id: "link-preview", width: 1200 },
];

/** The template `id`, of a recording's or a screenshot's. */
export const frameSizePreset = (id: string | null | undefined) =>
  FRAME_SIZE_PRESETS.find((preset) => preset.id === id) ??
  SCREENSHOT_SIZE_PRESETS.find((preset) => preset.id === id);

/** The platform a preset is for, which heads its group in a list. */
export const frameSizePresetPlatform = (id: FrameSizePresetId): string => {
  if (id.startsWith("youtube")) return t("editor-frame-platform-youtube");
  if (id.startsWith("tiktok")) return t("editor-frame-platform-tiktok");
  if (id.startsWith("instagram")) return t("editor-frame-platform-instagram");
  if (id.startsWith("linkedin")) return t("editor-frame-platform-linkedin");
  if (id.startsWith("facebook")) return t("editor-frame-platform-facebook");
  if (id.startsWith("pinterest")) return t("editor-frame-platform-pinterest");
  if (id === "link-preview") return t("editor-frame-platform-web");
  return t("editor-frame-platform-x");
};

/** A preset's name: its platform and what it is for. */
export const frameSizePresetName = (id: FrameSizePresetId): string => {
  switch (id) {
    case "youtube-1080":
      return t("editor-frame-size-youtube-1080");
    case "youtube-1440":
      return t("editor-frame-size-youtube-1440");
    case "youtube-2160":
      return t("editor-frame-size-youtube-2160");
    case "youtube-shorts":
      return t("editor-frame-size-youtube-shorts");
    case "tiktok":
      return t("editor-frame-size-tiktok");
    case "instagram-reels":
      return t("editor-frame-size-instagram-reels");
    case "instagram-portrait":
      return t("editor-frame-size-instagram-portrait");
    case "instagram-square":
      return t("editor-frame-size-instagram-square");
    case "linkedin-landscape":
      return t("editor-frame-size-linkedin-landscape");
    case "linkedin-square":
      return t("editor-frame-size-linkedin-square");
    case "linkedin-portrait":
      return t("editor-frame-size-linkedin-portrait");
    case "linkedin-vertical":
      return t("editor-frame-size-linkedin-vertical");
    case "x-landscape":
      return t("editor-frame-size-x-landscape");
    case "x-vertical":
      return t("editor-frame-size-x-vertical");
    case "youtube-thumbnail":
      return t("editor-frame-size-youtube-thumbnail");
    case "youtube-thumbnail-4k":
      return t("editor-frame-size-youtube-thumbnail-4k");
    case "tiktok-photo":
      return t("editor-frame-size-tiktok-photo");
    case "instagram-tall-portrait":
      return t("editor-frame-size-instagram-tall-portrait");
    case "instagram-story":
      return t("editor-frame-size-instagram-story");
    case "linkedin-image-landscape":
      return t("editor-frame-size-linkedin-image-landscape");
    case "x-image-landscape":
      return t("editor-frame-size-x-image-landscape");
    case "x-image-portrait":
      return t("editor-frame-size-x-image-portrait");
    case "facebook-post":
      return t("editor-frame-size-facebook-post");
    case "pinterest-pin":
      return t("editor-frame-size-pinterest-pin");
    case "link-preview":
      return t("editor-frame-size-link-preview");
  }
};
