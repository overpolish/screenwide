// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { HotkeyField } from "../hotkey-field/hotkey-field";

import { modifierDescription, modifierKeys, modifierTitle } from "./glide-rows";

import type { GlideSettings } from "../../../bindings/GlideSettings";

// The keys held while dragging a window to steer Glide, one each. Taking a
// key another one already has swaps the two, so no two ever share a key.
export function GlideKeys({
  defaults,
  isDisabled,
  onCaptureChange,
  onChange,
  settings,
}: {
  isDisabled: boolean;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
  defaults?: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  return (
    <>
      {modifierKeys.map((key) => {
        const otherKeys = modifierKeys.filter((candidate) => candidate !== key);
        const title = modifierTitle(key);
        return (
          <ControlRow
            description={modifierDescription(key)}
            key={key}
            title={title}
          >
            {(controlProps) => (
              <HotkeyField
                aria-describedby={controlProps["aria-describedby"]}
                aria-label={title}
                captureMode="single-control"
                defaultValue={defaults?.[key]}
                isClearable={false}
                isDisabled={isDisabled}
                onCaptureChange={onCaptureChange}
                onChange={(value) => {
                  if (value === null) return;
                  const collidingKey = otherKeys.find(
                    (candidate) => settings[candidate] === value,
                  );
                  update({
                    [key]: value,
                    ...(collidingKey ? { [collidingKey]: settings[key] } : {}),
                  });
                }}
                value={settings[key]}
              />
            )}
          </ControlRow>
        );
      })}
    </>
  );
}
