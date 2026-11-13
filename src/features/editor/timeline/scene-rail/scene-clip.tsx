// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";
import { RailClip, type RailClipProps } from "../rail/rail-clip";
import { type Span } from "../timeline-scale";

import type { SceneDrawing } from "../../glyphs/scene-layout-drawing";

// One scene over the stretch it frames (`rail-clip.tsx`), drawn as its
// layout and named.

export type TimelineScene = Span & {
  drawing: SceneDrawing;
  id: string;
  /** What its layout is called. */
  name: string;
  /** Where choosing it brings the playhead, in recorded seconds: once it has
   * arrived. Its start where not given. */
  landsAt?: number;
};

export type SceneClipProps = RailClipProps & { scene: TimelineScene };

export function SceneClip({ scene, ...clip }: SceneClipProps) {
  return (
    <RailClip
      {...clip}
      glyph={<SceneLayoutGlyph drawing={scene.drawing} />}
      label={t("editor-timeline-scene", { layout: scene.name })}
      name={scene.name}
      span={scene}
    />
  );
}
