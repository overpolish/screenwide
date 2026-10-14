// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import base from "../../../../../src-tauri/crates/captions/templates/base.json";
import index from "../../../../../src-tauri/crates/captions/templates/index.json";
import motionPresets from "../../../../../src-tauri/crates/captions/templates/motion-presets.json";
import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";

import type { CaptionMotion } from "../../../../bindings/CaptionMotion";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplates } from "../../recording/captions/use-caption-templates";

/** A manifest's options, its motion written whole or as a preset's name. */
type Manifest = {
  id: string;
  style: Partial<Omit<CaptionStyle, "motion">> & {
    motion?: CaptionMotion | string;
  };
};

const presets = motionPresets.presets as Record<string, CaptionMotion>;

/** Every bundled template's manifest, by path. */
const manifests = import.meta.glob<Manifest>(
  "/src-tauri/crates/captions/templates/*/template.json",
  { eager: true, import: "default" },
);

/** The templates as the app ships them, read from the same manifests, for
 * stories, which have no app to ask: each one's options over the base, as
 * `parse_manifest` sets them. */
export const STORY_CAPTION_STYLES: CaptionStyle[] = index.templates.map(
  (id) => {
    const manifest = Object.values(manifests).find((each) => each.id === id);
    if (!manifest) throw new Error(`No bundled manifest for "${id}"`);
    const { motion, ...options } = manifest.style;
    return {
      ...(base.style as Omit<CaptionStyle, "template">),
      ...options,
      ...(motion === undefined
        ? {}
        : { motion: typeof motion === "string" ? presets[motion] : motion }),
      template: id,
    };
  },
);

export const STORY_CAPTION_TEMPLATES: CaptionTemplates = {
  defaultLayout: {},
  defaultSettings: DEFAULT_CAPTION_SETTINGS,
  defaultTemplate: index.default,
  shipped: STORY_CAPTION_STYLES,
  styles: STORY_CAPTION_STYLES,
};
