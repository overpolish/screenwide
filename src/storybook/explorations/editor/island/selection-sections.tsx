// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { type SceneClip, ZOOMS } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { SectionLine } from "./island-parts";
import { PresetChoice } from "./preset-choice";

// What the island holds for a beat, a scene or a pause on the strip: the
// actions that sit in its row, and the choices that hang beneath it. How a
// beat is framed is the scene over it, not a setting of the beat's.

export function BeatActions({ editor }: { editor: Editor }) {
  return (
    <ConfirmActionButton
      confirmLabel="Confirm Cut"
      icon={<Scissors aria-hidden />}
      label="Cut it and its time"
      onConfirm={editor.remove}
      variant="ghost"
    />
  );
}

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
