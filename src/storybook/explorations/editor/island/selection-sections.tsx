// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { type SceneClip, ZOOMS } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { SectionLine } from "./island-parts";
import { PresetChoice } from "./preset-choice";

// What the island holds for a scene on the strip: the choices that hang
// beneath its row.

export function SceneClipSection({
  clip,
  editor,
}: {
  clip: SceneClip;
  editor: Editor;
}) {
  return (
    <div className="flex flex-col gap-2">
      <PresetChoice
        onChange={(preset) => {
          editor.updateScene(clip.id, { preset });
        }}
        value={clip.preset}
      />
      {clip.preset === "zoom" ? (
        <SectionLine label="How far in">
          <PillGroup
            aria-label="How far in"
            items={ZOOMS.map((zoom) => ({
              ariaLabel: `${String(zoom)} times`,
              icon: (
                <span className="text-label tabular-nums">{`${String(zoom)}×`}</span>
              ),
              id: String(zoom),
            }))}
            onSelectionChange={(id) => {
              editor.updateScene(clip.id, { zoom: Number(id) });
            }}
            selected={String(clip.zoom)}
          />
        </SectionLine>
      ) : null}
    </div>
  );
}
