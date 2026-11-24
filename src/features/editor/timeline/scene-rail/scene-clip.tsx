// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";
import { useFrameFormats } from "../../recording/formats/frame-formats-context";
import { useShownElsewhere } from "../../recording/formats/use-shown-elsewhere";
import { RailClip, type RailClipProps } from "../rail/rail-clip";
import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

import type { SceneDrawing } from "../../glyphs/scene-layout-drawing";

// One scene over the stretch it frames (`rail-clip.tsx`), drawn as its
// layout and named.

export type TimelineScene = Span & {
  drawing: SceneDrawing;
  id: string;
  /** What its layout is called. */
  name: string;
};

export type SceneClipProps = RailClipProps & { scene: TimelineScene };

export function SceneClip({ scene, ...clip }: SceneClipProps) {
  const differs = useFrameFormats()?.snapshot.differs;
  const shownElsewhere = useShownElsewhere();
  const id = baseOf(scene.id);
  const ghostOf = shownElsewhere(id);
  const isMarked = differs?.ids.includes(id) ?? false;
  return (
    <RailClip
      {...clip}
      glyph={<SceneLayoutGlyph drawing={scene.drawing} />}
      isGhost={ghostOf !== null}
      isMarked={isMarked}
      label={
        ghostOf !== null
          ? t("editor-timeline-scene-ghost", {
              formats: ghostOf,
              layout: scene.name,
            })
          : isMarked
            ? t("editor-timeline-scene-differs", { layout: scene.name })
            : t("editor-timeline-scene", { layout: scene.name })
      }
      menuTarget={id}
      name={scene.name}
      span={scene}
    />
  );
}
