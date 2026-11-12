// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert } from "lucide-react";
import { useListFormatter } from "react-aria";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { hotkeyKeys } from "../../../lib/hotkey";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";
import { Text } from "../../../primitives/text/text";
import { SettingsGroup } from "../settings-group";
import { SettingsReveal } from "../settings-reveal";

import { HotkeyField } from "./hotkey-field/hotkey-field";
import { shortcutValues } from "./shortcut-entry";

import type { ShortcutConflict } from "./shortcut-conflicts";
import type { ShortcutEntry } from "./shortcut-entry";

// Every shortcut, grouped by feature, each set in its field, or shown as
// its keys where it is built into a tool. Pointing at a row rings its key on
// the map; choosing one (pressing it, or moving into its field) marks it as
// the one a click on the map gives a key to. A shortcut that shares its keys
// with another says which, and offers to take them from it.

function ClashLine({ others }: { others: readonly ShortcutEntry[] }) {
  const list = useListFormatter({ type: "conjunction" });
  return (
    <div className="flex items-center gap-inline pb-inset-sm">
      <CircleAlert aria-hidden className="size-icon-sm shrink-0 text-error" />
      <Text className="grow" variant="caption">
        {t("settings-shortcuts-clash", {
          others: list.format(others.map((other) => other.label)),
        })}
      </Text>
      <Button
        className="shrink-0"
        onPress={() => {
          for (const other of others) if (!other.isFixed) other.onChange(null);
        }}
        variant="ghost"
      >
        {t("settings-shortcuts-clash-remove", { shortcut: others[0].label })}
      </Button>
    </div>
  );
}

export function ShortcutList({
  clashes,
  conflictOf,
  entries,
  onCaptureChange,
  onPoint,
  onSelect,
  selectedId,
}: {
  clashes: ReadonlyMap<string, ShortcutEntry[]>;
  conflictOf: (
    entry: ShortcutEntry,
  ) => (value: string) => ShortcutConflict | null;
  entries: readonly ShortcutEntry[];
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onPoint: (id: string | null) => void;
  onSelect: (id: string) => void;
  selectedId: string | null;
}) {
  const groups = [...new Set(entries.map((entry) => entry.group))];
  return groups.map((group) => {
    const rows = entries.filter((entry) => entry.group === group);
    return (
      <SettingsGroup
        description={
          rows.every((entry) => entry.isOff)
            ? t("settings-shortcuts-feature-off", { feature: group })
            : undefined
        }
        key={group}
        title={group}
      >
        {rows.map((entry) => {
          const others = clashes.get(entry.id) ?? [];
          // Pointer and focus events only follow the row for the map; the
          // field inside is what is pressed.
          return (
            <div
              className={cn(
                "-mx-inset-sm flex flex-col rounded-control px-inset-sm transition-colors",
                entry.id === selectedId && "bg-selected",
              )}
              data-shortcut-id={entry.id}
              key={entry.id}
              onFocus={() => {
                onSelect(entry.id);
              }}
              onPointerDown={() => {
                onSelect(entry.id);
              }}
              onPointerEnter={() => {
                onPoint(entry.id);
              }}
              onPointerLeave={() => {
                onPoint(null);
              }}
            >
              <ControlRow description={entry.description} title={entry.label}>
                {(controlProps) =>
                  entry.isFixed ? (
                    <span
                      {...controlProps}
                      className="flex flex-wrap justify-end gap-separate"
                      role="group"
                    >
                      {shortcutValues(entry).map((value) => (
                        <Shortcut key={value}>
                          {hotkeyKeys(value).map((key) => (
                            <Keyboard key={key}>{key}</Keyboard>
                          ))}
                        </Shortcut>
                      ))}
                    </span>
                  ) : (
                    <HotkeyField
                      aria-describedby={controlProps["aria-describedby"]}
                      aria-label={entry.label}
                      aria-labelledby={controlProps["aria-labelledby"]}
                      captureMode={entry.captureMode}
                      conflictOf={conflictOf(entry)}
                      defaultValue={entry.defaultValue}
                      isClearable={entry.isClearable}
                      isDisabled={entry.isDisabled}
                      onCaptureChange={onCaptureChange}
                      onChange={entry.onChange}
                      value={entry.value}
                    />
                  )
                }
              </ControlRow>
              <SettingsReveal>
                {others.length > 0 ? <ClashLine others={others} /> : null}
              </SettingsReveal>
            </div>
          );
        })}
      </SettingsGroup>
    );
  });
}
