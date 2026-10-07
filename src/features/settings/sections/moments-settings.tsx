// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Button } from "../../../components/base/button/button";
import { GroupBox } from "../../../components/base/group-box/group-box";
import { TextField } from "../../../components/base/input-fields/text-field";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";

import { MomentColorGrid } from "./moment-color-grid";
import { VoiceNoteSettings } from "./voice-note-settings";

import type { MomentSettingsState } from "./use-moment-settings";
import type { MomentKind } from "../types";

/** A kind's name as it is typed, kept to the field until it is left or
 * Return is pressed, so the settings file is not written per keystroke. A
 * name cleared away goes back to the one it had. */
function KindNameField({
  isDisabled,
  kind,
  onCommit,
}: {
  isDisabled: boolean;
  kind: MomentKind;
  onCommit: (name: string) => void;
}) {
  const [draft, setDraft] = useState(kind.name);
  const commit = () => {
    const name = draft.trim();
    if (name && name !== kind.name) onCommit(name);
    else setDraft(kind.name);
  };
  return (
    <TextField
      aria-label={`${kind.name} name`}
      isDisabled={isDisabled}
      onBlur={commit}
      onChange={setDraft}
      onKeyDown={(event) => {
        if (event.key === "Enter") commit();
      }}
      value={draft}
    />
  );
}

export function MomentsSettingsPanel({
  moments,
  onCaptureChange,
}: {
  moments: MomentSettingsState;
  onCaptureChange: (capturing: boolean) => Promise<void>;
}) {
  const { isSaving, preview, save, settings } = moments;
  if (!settings) return null;
  const { kinds } = settings;
  const withChange = (id: string, changes: Partial<MomentKind>) => ({
    ...settings,
    kinds: kinds.map((kind) =>
      kind.id === id ? { ...kind, ...changes } : kind,
    ),
  });
  const update = (id: string, changes: Partial<MomentKind>) => {
    void save(withChange(id, changes));
  };
  return (
    <div className="gap-layout flex flex-col">
      <VoiceNoteSettings
        isSaving={isSaving}
        onChange={(next) => {
          void save(next);
        }}
        settings={settings}
      />
      {kinds.map((kind) => (
        <GroupBox key={kind.id} title={kind.name}>
          <ControlRow title="Name">
            {() => (
              <KindNameField
                isDisabled={isSaving}
                key={kind.name}
                kind={kind}
                onCommit={(name) => {
                  update(kind.id, { name });
                }}
              />
            )}
          </ControlRow>
          <ControlRow description="Works while recording." title="Shortcut">
            {(controlProps) => (
              <HotkeyField
                aria-describedby={controlProps["aria-describedby"]}
                aria-label={`${kind.name} shortcut`}
                isDisabled={isSaving}
                onCaptureChange={onCaptureChange}
                onChange={(shortcut) => {
                  update(kind.id, { shortcut });
                }}
                value={kind.shortcut}
              />
            )}
          </ControlRow>
          <ControlRow title="Colour">
            {() => (
              <MomentColorGrid
                color={kind.color}
                customColor={kind.customColor}
                isDisabled={isSaving}
                onChange={(change) => {
                  update(kind.id, change);
                }}
                onPreview={(change) => {
                  preview(withChange(kind.id, change));
                }}
              />
            )}
          </ControlRow>
          <div className="gap-control flex justify-end">
            <Button
              isDisabled={isSaving}
              onPress={() => {
                void save({
                  ...settings,
                  kinds: kinds.filter((other) => other.id !== kind.id),
                });
              }}
            >
              Remove
            </Button>
          </div>
        </GroupBox>
      ))}
    </div>
  );
}
