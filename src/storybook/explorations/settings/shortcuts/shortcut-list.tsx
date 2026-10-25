// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert } from "lucide-react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { HotkeyField } from "../../../../features/settings/hotkey-field/hotkey-field";
import { SettingsGroup } from "../../../../features/settings/settings-group";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { Text } from "../../../../primitives/text/text";

import type { ShortcutEntry } from "./shortcut-model";

// The shortcuts of the map's scope, grouped by feature, each set by typing
// it into its field. Pointing at a row lights its key on the map; choosing
// one (pressing it, or tabbing into its field) marks it as the one a click
// on the map gives a new key to. A shortcut that shares its keys with
// another says which, and offers to take them from it.

export function ShortcutList({
  clashes,
  entries,
  onCaptureChange,
  onPreview,
  onSelect,
  selectedId,
}: {
  clashes: ReadonlyMap<string, ShortcutEntry[]>;
  entries: ShortcutEntry[];
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onPreview: (id: string | null) => void;
  onSelect: (id: string) => void;
  selectedId: string | null;
}) {
  const groups = [...new Set(entries.map((entry) => entry.group))];
  return groups.map((group) => {
    const rows = entries.filter((entry) => entry.group === group);
    const isOff = rows.every((entry) => entry.isOff);
    return (
      <SettingsGroup
        description={
          isOff ? `Turn on ${rows[0].group} in Tools to use these.` : undefined
        }
        key={group}
        title={group}
      >
        {rows.map((entry) => {
          const others = clashes.get(entry.id) ?? [];
          return (
            <div
              className={cn(
                "-mx-inset-sm flex flex-col gap-inline rounded-control px-inset-sm transition-colors",
                entry.id === selectedId && "bg-selected",
              )}
              data-entry-id={entry.id}
              key={entry.id}
              onFocus={() => {
                onSelect(entry.id);
              }}
              onPointerDown={() => {
                onSelect(entry.id);
              }}
              onPointerEnter={() => {
                onPreview(entry.id);
              }}
              onPointerLeave={() => {
                onPreview(null);
              }}
            >
              <ControlRow description={entry.description} title={entry.label}>
                {(controlProps) => (
                  <HotkeyField
                    aria-describedby={controlProps["aria-describedby"]}
                    aria-label={entry.label}
                    captureMode={entry.captureMode}
                    isClearable={entry.kind === "combo"}
                    isDisabled={entry.isOff}
                    onCaptureChange={onCaptureChange}
                    onChange={entry.onChange}
                    value={entry.value}
                  />
                )}
              </ControlRow>
              {others.length > 0 ? (
                <div className="flex items-center gap-inline pb-inset-sm">
                  <CircleAlert
                    aria-hidden
                    className="size-icon-sm shrink-0 text-error"
                  />
                  <Text className="grow" variant="caption">
                    {`Same keys as ${others.map((other) => other.label).join(" and ")}.`}
                  </Text>
                  <Button
                    className="shrink-0"
                    onPress={() => {
                      for (const other of others) other.onChange(null);
                    }}
                    variant="ghost"
                  >
                    {`Remove from ${others[0].label}`}
                  </Button>
                </div>
              ) : null}
            </div>
          );
        })}
      </SettingsGroup>
    );
  });
}
