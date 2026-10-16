// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import base from "../../../../../src-tauri/crates/text/templates/base.json";
import index from "../../../../../src-tauri/crates/text/templates/index.json";
import motionPresets from "../../../../../src-tauri/crates/text/templates/motion-presets.json";
import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";

import type { TextMotion } from "../../../../bindings/TextMotion";
import type { TextStyle } from "../../../../bindings/TextStyle";
import type { TextTemplates } from "../../recording/captions/use-text-templates";

/** A manifest's options, its motion written whole or as a preset's name. */
type Manifest = {
  id: string;
  name: string;
  style: Partial<Omit<TextStyle, "motion">> & {
    motion?: TextMotion | string;
  };
};

const presets = motionPresets.presets as Record<string, TextMotion>;

/** Every bundled template's manifest, by path. */
const manifests = import.meta.glob<Manifest>(
  "/src-tauri/crates/text/templates/*/template.json",
  { eager: true, import: "default" },
);

/** The templates as the app ships them, read from the same manifests, for
 * stories, which have no app to ask: each one's options over the base, as
 * `parse_manifest` sets them. */
export const STORY_TEXT_STYLES: TextStyle[] = index.templates.map((id) => {
  const manifest = Object.values(manifests).find((each) => each.id === id);
  if (!manifest) throw new Error(`No bundled manifest for "${id}"`);
  const { motion, ...options } = manifest.style;
  return {
    ...(base.style as Omit<TextStyle, "template">),
    ...options,
    ...(motion === undefined
      ? {}
      : { motion: typeof motion === "string" ? presets[motion] : motion }),
    template: id,
  };
});

/** The style template `id` ships with, for a story or test that needs one
 * template in particular. */
export const storyTextStyle = (id: string): TextStyle => {
  const style = STORY_TEXT_STYLES.find((each) => each.template === id);
  if (!style) throw new Error(`No bundled template "${id}"`);
  return style;
};

export const STORY_TEXT_TEMPLATES: TextTemplates = {
  defaultLayout: {},
  defaultSettings: DEFAULT_CAPTION_SETTINGS,
  defaultTemplate: index.default,
  names: Object.fromEntries(
    Object.values(manifests).map((manifest) => [manifest.id, manifest.name]),
  ),
  shipped: STORY_TEXT_STYLES,
  sources: Object.fromEntries(
    STORY_TEXT_STYLES.map((style) => [style.template, "bundled"]),
  ),
  styles: STORY_TEXT_STYLES,
};
