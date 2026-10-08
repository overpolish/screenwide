// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Trash2 } from "lucide-react";
import { useState } from "react";

import { GroupBox } from "../../../components/base/group-box/group-box";
import { TextField } from "../../../components/base/input-fields/text-field";
import { Text } from "../../../components/base/text/text";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";

import { KindColorPicker } from "./kind-color-picker";
import { VoiceNoteSettings } from "./voice-note-settings";

import type { MomentSettingsState } from "./use-moment-settings";
import type { TranscriptionControls } from "./use-transcription";
import type { MomentKind } from "../../../bindings/MomentKind";

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
      className="min-w-0 grow"
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
  transcription,
}: {
  moments: MomentSettingsState;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  transcription: TranscriptionControls;
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
      {/* One line per moment: everything one is, editable at a glance. */}
      <GroupBox title="Moments">
        {kinds.map((kind) => (
          <div className="gap-control flex items-center" key={kind.id}>
            <KindColorPicker
              color={kind.color}
              customColor={kind.customColor}
              isDisabled={isSaving}
              name={kind.name}
              onChange={(change) => {
                update(kind.id, change);
              }}
              onPreview={(change) => {
                preview(withChange(kind.id, change));
              }}
            />
            <KindNameField
              isDisabled={isSaving}
              key={kind.name}
              kind={kind}
              onCommit={(name) => {
                update(kind.id, { name });
              }}
            />
            <HotkeyField
              aria-label={`${kind.name} shortcut`}
              isDisabled={isSaving}
              onCaptureChange={onCaptureChange}
              onChange={(shortcut) => {
                update(kind.id, { shortcut });
              }}
              value={kind.shortcut}
            />
            <ConfirmActionButton
              armedIcon={<Check />}
              armedLabel={`Confirm removing ${kind.name}`}
              idleIcon={<Trash2 />}
              idleLabel={`Remove ${kind.name}`}
              isDisabled={isSaving}
              onConfirm={() => {
                void save({
                  ...settings,
                  kinds: kinds.filter((other) => other.id !== kind.id),
                });
              }}
              variant="icon"
            />
          </div>
        ))}
        <Text variant="footnote">
          Press a moment's shortcut while recording to place it.
        </Text>
      </GroupBox>
      <VoiceNoteSettings
        isSaving={isSaving}
        onChange={(next) => {
          void save(next);
        }}
        settings={settings}
        transcription={transcription}
      />
    </div>
  );
}
