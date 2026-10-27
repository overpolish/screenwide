// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { cn } from "../../../../lib/styling";
import { IconButton } from "../../../../primitives/button/icon-button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { type Backdrop, BACKDROPS, SCENE_PRESETS } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { CheckList, SectionLine } from "./island-parts";
import { PresetDrawing } from "./preset-choice";

// What the island holds for a mode picked from the toolbar. Each acts on
// the picture as it is changed, so there is nothing to apply.

export function FrameSection({ editor }: { editor: Editor }) {
  return (
    <SectionLine label="Background">
      <PillGroup
        aria-label="Background"
        items={BACKDROPS.map(({ background, id, label }) => ({
          ariaLabel: label,
          icon: (
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
        }))}
        onSelectionChange={(id) => {
          editor.setBackdrop(id as Backdrop);
        }}
        selected={editor.backdrop}
      />
    </SectionLine>
  );
}

export function PointerSection({ editor }: { editor: Editor }) {
  const [isSmooth, setIsSmooth] = useState(true);
  return (
    <div className="w-52">
      <CheckList
        aria-label="Pointer"
        items={[
          { id: "ripple", label: "Show clicks" },
          { id: "smooth", label: "Smooth movement" },
        ]}
        onToggle={(id) => {
          if (id === "ripple")
            editor.setEffects({
              ...editor.effects,
              ripple: !editor.effects.ripple,
            });
          else setIsSmooth(!isSmooth);
        }}
        selected={[
          ...(editor.effects.ripple ? ["ripple"] : []),
          ...(isSmooth ? ["smooth"] : []),
        ]}
      />
    </div>
  );
}

/** A scene to add at the playhead, as each arrangement's circle: pressing
 * one adds it there, over the beat under the playhead. */
export function SceneSection({ editor }: { editor: Editor }) {
  return (
    <div className="flex gap-1">
      {SCENE_PRESETS.map(({ id, label }) => (
        <IconButton
          aria-label={`Add ${label}`}
          key={id}
          onPress={() => {
            editor.addScene(id);
          }}
        >
          <PresetDrawing id={id} />
        </IconButton>
      ))}
    </div>
  );
}
