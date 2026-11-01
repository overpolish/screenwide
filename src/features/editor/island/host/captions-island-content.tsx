// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  handLayout,
  isSameLayout,
  layoutOf,
  withLayout,
} from "../../recording/captions/caption-layout";
import { defaultCaptions } from "../../recording/captions/use-text-templates";
import { captionsIsland } from "../captions/captions-island";

import type { TextLayout } from "../../../../bindings/TextLayout";
import type { CaptionsPatch } from "../../recording/captions/recording-captions";
import type { ToolPanelPatch } from "../../tool-panels/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tool-panels/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";
import type { IslandTranscript } from "../island-transcript";

// The recording's captions on the Captions island, changed through the
// editor's `ToolPanelPatch` asks. Their look holds the layout options, the
// ones set by hand kept apart so another template keeps them (`handLayout`).
// Reset and Set as default take the look, the layout and the rules
// together: back to what new captions start as, or making these that.

const NO_LAYOUT: TextLayout = {};

/** Whether every field `a` holds is the same in `b`. */
const isSame = <T extends object>(a: T, b: T) =>
  (Object.keys(a) as (keyof T)[]).every(
    (key) => JSON.stringify(a[key]) === JSON.stringify(b[key]),
  );

export function captionsIslandContent({
  change,
  snapshot,
  transcript,
}: {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
  transcript: IslandTranscript;
}): EditorIslandContent | null {
  const { annotationColors, captions, textTemplates: templates } = snapshot;
  if (!captions || !templates) return null;
  const ask = (patch: CaptionsPatch) => {
    change({ captions: patch });
  };
  const { settings, style } = captions;
  const layout = captions.layout ?? NO_LAYOUT;
  const template = templates.styles.find(
    (each) => each.template === style.template,
  );
  const defaults = defaultCaptions(templates);
  // The look alone: the layout options the saved look carries are the ones
  // the template ships with, so the captions' own are set aside.
  const look = template ? withLayout(layoutOf(template), style) : style;
  const isDefault =
    isSame(style, defaults.style) &&
    isSameLayout(layout, defaults.layout) &&
    isSame(settings, defaults.settings);
  return captionsIsland({
    canReset: !isDefault,
    canSetDefault: !isDefault,
    enabled: captions.enabled,
    hiddenCount: captions.hidden?.length ?? 0,
    look: style,
    onEnabledChange: (enabled) => {
      ask({ enabled });
    },
    onLayoutChange: (next) => {
      ask({
        layout: handLayout({ ...layout, ...next }, template),
        style: next,
      });
    },
    onLookChange: (patch) => {
      // A template chosen brings its own look, but for the layout options
      // set by hand.
      ask({
        style:
          patch.template !== undefined && patch.template !== style.template
            ? withLayout(layout, { ...style, ...patch })
            : patch,
      });
    },
    onRemoveColor: (color) => {
      change({ removeAnnotationColor: color });
    },
    onReset: () => {
      ask({
        layout: defaults.layout,
        settings: defaults.settings,
        style: defaults.style,
      });
    },
    onRestoreHidden: () => {
      ask({ hidden: [] });
    },
    onSaveColor: (color) => {
      change({ saveAnnotationColor: color });
    },
    onSetDefault: () => {
      change({ captionDefaults: { layout, settings, style: look } });
    },
    onSettingsChange: (next) => {
      ask({ settings: next });
    },
    savedColors: annotationColors,
    settings,
    templates,
    transcript,
  });
}
