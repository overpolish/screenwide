// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { IslandChoice } from "../../../../primitives/island/island-choice";
import { IslandListRow } from "../../../../primitives/island/island-controls";
import { Text } from "../../../../primitives/text/text";
import { type SceneClip, ZOOMS } from "../editor-data";
import { ownLength, speedLabel, speedOf } from "../model/speed";
import { DEAD_AIR_S } from "../model/timeline-model";
import { type Editor } from "../model/use-editor";

import { IslandConfirmButton } from "./island-confirm-button";
import { CheckRow, SectionLine } from "./island-parts";
import { PresetChoice } from "./preset-choice";

// What the island holds for a beat, a scene or a pause on the strip: the
// actions that sit in its row, and the choices that hang beneath it. How a
// beat is framed is the scene over it, not a setting of the beat's.

export function BeatActions({ editor }: { editor: Editor }) {
  return (
    <IslandConfirmButton
      confirmLabel="Confirm Cut"
      label="Cut it and its time"
      onConfirm={editor.remove}
    >
      <Scissors aria-hidden className="size-icon" />
    </IslandConfirmButton>
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
          <IslandChoice
            items={ZOOMS.map((zoom) => ({
              content: (
                <span className="text-label tabular-nums">{`${String(zoom)}×`}</span>
              ),
              id: String(zoom),
              label: `${String(zoom)} times`,
            }))}
            label="How far in"
            onChange={(id) => {
              editor.updateScene(clip.id, { zoom: Number(id) });
            }}
            value={String(clip.zoom)}
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
      <CheckRow
        isChosen={!setting}
        onPress={() => {
          editor.setPause(id, null);
        }}
      >
        Squeeze with the rest
      </CheckRow>
      <CheckRow
        isChosen={setting?.kind === "keep"}
        onPress={() => {
          editor.setPause(id, { kind: "keep" });
        }}
      >
        Keep as recorded
      </CheckRow>
      {own === null ? null : (
        <CheckRow isChosen onPress={() => undefined}>
          {retimedLabel(
            pause.end - pause.start,
            ownLength(pause, own, editor.recording.look),
          )}
        </CheckRow>
      )}
      <IslandListRow
        onPress={() => {
          editor.trimToBreath(id);
        }}
      >
        Trim to a breath
      </IslandListRow>
      {pause.still > DEAD_AIR_S && deadAir > 1 ? (
        <IslandListRow onPress={editor.trimDeadAir}>
          {`Trim all ${String(deadAir)} dead air pauses`}
        </IslandListRow>
      ) : null}
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
