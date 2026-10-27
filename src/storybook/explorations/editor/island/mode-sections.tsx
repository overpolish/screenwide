// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { cn } from "../../../../lib/styling";
import { IslandChoice } from "../../../../primitives/island/island-choice";
import { IslandButton } from "../../../../primitives/island/island-controls";
import { BACKDROPS, SCENE_PRESETS } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { CheckRow, SectionLine } from "./island-parts";
import { PresetDrawing } from "./preset-choice";

// What the island holds for a mode picked from the toolbar. Each acts on
// the picture as it is changed, so there is nothing to apply.

export function FrameSection({ editor }: { editor: Editor }) {
  return (
    <SectionLine label="Background">
      <IslandChoice
        items={BACKDROPS.map(({ background, id, label }) => ({
          content: (
            <span
              className={cn(
                "size-5 rounded-full",
                id === "none"
                  ? "border border-dashed border-fg-tertiary"
                  : "ring-1 ring-divider",
              )}
              style={{ background }}
            />
          ),
          id,
          label,
        }))}
        label="Background"
        onChange={editor.setBackdrop}
        value={editor.backdrop}
      />
    </SectionLine>
  );
}

export function PointerSection({ editor }: { editor: Editor }) {
  const [isSmooth, setIsSmooth] = useState(true);
  return (
    <div className="flex w-52 flex-col">
      <CheckRow
        isChosen={editor.effects.ripple}
        onPress={() => {
          editor.setEffects({
            ...editor.effects,
            ripple: !editor.effects.ripple,
          });
        }}
      >
        Show clicks
      </CheckRow>
      <CheckRow
        isChosen={isSmooth}
        onPress={() => {
          setIsSmooth(!isSmooth);
        }}
      >
        Smooth movement
      </CheckRow>
    </div>
  );
}

/** A scene to add at the playhead, as each arrangement's circle: pressing
 * one adds it there, over the beat under the playhead. */
export function SceneSection({ editor }: { editor: Editor }) {
  return (
    <div className="flex gap-1">
      {SCENE_PRESETS.map(({ id, label }) => (
        <IslandButton
          aria-label={`Add ${label}`}
          key={id}
          onPress={() => {
            editor.addScene(id);
          }}
        >
          <PresetDrawing id={id} />
        </IslandButton>
      ))}
    </div>
  );
}
