// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import { useState } from "react";
import { Button as AriaButton } from "react-aria-components";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { hotkeyKeys } from "../../../lib/hotkey";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { TextField } from "../../../primitives/input-fields/text-field";
import { keyLabel } from "../../../primitives/keyboard/key-label";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";
import { SettingsGroup } from "../settings-group";
import { hotkeyTriggerClassName } from "../shortcuts/hotkey-field/hotkey-trigger-style";

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
      aria-label={t("settings-moment-name", { moment: kind.name })}
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

/** A kind's keys, which are set with every other shortcut in the
 * Shortcuts pane; pressing them takes you to its row there. It looks as
 * that row's field does, keycaps in a raised field. */
function KindShortcutLink({
  kind,
  onOpen,
}: {
  kind: MomentKind;
  onOpen: () => void;
}) {
  const keys = hotkeyKeys(kind.shortcut);
  return (
    <AriaButton
      aria-label={t("settings-moment-shortcut-open", {
        keys: keys.map(keyLabel).join(" + ") || t("hotkey-field-not-set"),
        moment: kind.name,
      })}
      className={cn(hotkeyTriggerClassName, "shrink-0")}
      onPress={onOpen}
    >
      {keys.length > 0 ? (
        <Shortcut>
          {keys.map((key, index) => (
            <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
          ))}
        </Shortcut>
      ) : (
        <span className="truncate text-fg-tertiary">
          {t("hotkey-field-set")}
        </span>
      )}
    </AriaButton>
  );
}

export function MomentsSettingsPanel({
  moments,
  onOpenShortcut,
  transcription,
}: {
  moments: MomentSettingsState;
  /** Opens the Shortcuts pane on the row of the shortcut with this title. */
  onOpenShortcut: (title: string) => void;
  transcription: TranscriptionControls;
}) {
  const { addKind, isSaving, preview, save, settings } = moments;
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
    <div className="flex flex-col gap-group">
      {/* One line per moment: everything one is, editable at a glance. */}
      <SettingsGroup footnote={t("settings-moments-footnote")}>
        {kinds.map((kind) => (
          <div className="flex items-center gap-inline" key={kind.id}>
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
            <KindShortcutLink
              kind={kind}
              onOpen={() => {
                onOpenShortcut(
                  t("settings-shortcut-place-moment", { moment: kind.name }),
                );
              }}
            />
            <ConfirmActionButton
              confirmLabel={t("settings-moment-confirm-remove", {
                moment: kind.name,
              })}
              icon={<Trash2 />}
              isDisabled={isSaving}
              label={t("settings-moment-remove", { moment: kind.name })}
              onConfirm={() => {
                void save({
                  ...settings,
                  kinds: kinds.filter((other) => other.id !== kind.id),
                });
              }}
              variant="ghost"
            />
          </div>
        ))}
        {/* Added at the end of the list it adds to. */}
        <Button
          className="self-start"
          isDisabled={isSaving}
          onPress={addKind}
          variant="ghost"
        >
          {t("settings-add-moment")}
        </Button>
      </SettingsGroup>
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
