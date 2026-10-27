// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Text } from "../../../../primitives/text/text";
import { type SceneClip, ZOOMS } from "../editor-data";
import { ownLength, speedLabel, speedOf } from "../model/speed";
import { DEAD_AIR_S } from "../model/timeline-model";
import { type Editor } from "../model/use-editor";

import { CheckList, SectionLine } from "./island-parts";
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

export function PauseSection({ editor, id }: { editor: Editor; id: string }) {
  const pause = editor.map.pauses.find((item) => item.id === id);
  if (!pause) return null;
  const setting = editor.settings[id] as Editor["settings"][string] | undefined;
  const own = setting?.kind === "own" ? setting.seconds : null;
  const deadAir = editor.map.pauses.filter(
    ({ still }) => still > DEAD_AIR_S,
  ).length;
  return (
    <div className="flex w-56 flex-col">
      {/* How it plays: one of these, the last only once its end has been
          dragged to a speed of its own. */}
      <CheckList
        aria-label="How it plays"
        items={[
          { id: "squeeze", label: "Squeeze with the rest" },
          { id: "keep", label: "Keep as recorded" },
          ...(own === null
            ? []
            : [
                {
                  id: "own",
                  label: retimedLabel(
                    pause.end - pause.start,
                    ownLength(pause, own, editor.recording.look),
                  ),
                },
              ]),
        ]}
        onToggle={(choice) => {
          if (choice === "squeeze") editor.setPause(id, null);
          if (choice === "keep") editor.setPause(id, { kind: "keep" });
        }}
        selected={[
          setting?.kind === "own" ? "own" : (setting?.kind ?? "squeeze"),
        ]}
      />
      <ListBox
        aria-label="Trim"
        onAction={(action) => {
          if (action === "breath") editor.trimToBreath(id);
          else editor.trimDeadAir();
        }}
      >
        <ListBoxItem id="breath" showsSelection={false}>
          Trim to a breath
        </ListBoxItem>
        {pause.still > DEAD_AIR_S && deadAir > 1 ? (
          <ListBoxItem id="dead-air" showsSelection={false}>
            {`Trim all ${String(deadAir)} dead air pauses`}
          </ListBoxItem>
        ) : null}
      </ListBox>
      <Text
        as="p"
        className="px-inset-sm pt-1 text-fg-secondary"
        variant="caption"
      >
        Drag its end to speed it up or slow it down; ⌥-drag to trim it.
      </Text>
    </div>
  );
}

/** How a retimed pause plays, against real time. */
function retimedLabel(recorded: number, plays: number) {
  const speed = speedOf(recorded, plays);
  if (speed === null) return "Retimed, as recorded";
  return speed > 1
    ? `Retimed, ${speedLabel(speed)} faster`
    : `Retimed, ${speedLabel(speed)} speed`;
}
