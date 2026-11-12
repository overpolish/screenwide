// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert, CircleCheck, Info } from "lucide-react";
import { type ReactNode } from "react";

import { platformArg, t } from "../../../../i18n/i18n";
import { splitAtPlaceable } from "../../../../i18n/split-at-placeable";
import {
  type HotkeyModifier,
  hotkeyKeys,
  hotkeyValue,
  parseHotkey,
} from "../../../../lib/hotkey";
import { Keyboard, Shortcut } from "../../../../primitives/keyboard/keyboard";
import { Text } from "../../../../primitives/text/text";
import { type ReservedShortcut, reservedUse } from "../reserved-shortcuts";
import { isFixedScope, type ShortcutScope } from "../shortcut-conflicts";
import { shortcutValues, type ShortcutEntry } from "../shortcut-entry";

// The line under the map, saying in words what the map shows: everything
// the key under the pointer does, under any modifiers; what a click on a
// key did, or why it did not take; or how to use the map. It stays one
// line tall whatever it says, so the list under it never moves.

export type MapNotice = { text: string; tone: "done" | "refused" };

function Line({ children, icon }: { children: ReactNode; icon: ReactNode }) {
  return (
    <div
      aria-live="polite"
      className="flex h-control-height min-w-0 items-center gap-inline px-inset-sm"
    >
      {icon}
      <div className="flex min-w-0 items-center gap-separate overflow-hidden whitespace-nowrap">
        {children}
      </div>
    </div>
  );
}

function Keys({ value }: { value: string }) {
  return (
    <Shortcut>
      {hotkeyKeys(value).map((key) => (
        <Keyboard key={key}>{key}</Keyboard>
      ))}
    </Shortcut>
  );
}

function Use({ label, value }: { label: string; value: string }) {
  return (
    <span className="flex shrink-0 items-center gap-inline">
      <Keys value={value} />
      <Text as="span" variant="caption">
        {label}
      </Text>
    </span>
  );
}

/** Who already uses the combination and for what, in a sentence with its
 * keys set in it: “Most apps use ⇧⌘Z for Redo.” */
function Reserved({
  reserved,
  value,
}: {
  reserved: ReservedShortcut;
  value: string;
}) {
  const use = reserved.label();
  const [before, after] = splitAtPlaceable((keys) =>
    reserved.by === "system"
      ? t("settings-shortcuts-reserved-system", {
          keys,
          platform: platformArg(),
          use,
        })
      : t("settings-shortcuts-reserved-apps", { keys, use }),
  );
  return (
    <Text
      as="span"
      className="flex shrink-0 items-center gap-inline"
      variant="caption"
    >
      {before}
      <Keys value={value} />
      {after}
    </Text>
  );
}

const infoIcon = (
  <Info aria-hidden className="size-icon-sm shrink-0 text-fg-tertiary" />
);

export function KeyDetail({
  entries,
  hoveredKey,
  isMac,
  latched,
  notice,
  scope,
  selected,
}: {
  entries: readonly ShortcutEntry[];
  /** The code of the key under the pointer. */
  hoveredKey: string | null;
  isMac: boolean;
  latched: ReadonlySet<HotkeyModifier>;
  notice: MapNotice | null;
  scope: ShortcutScope;
  selected: ShortcutEntry | null;
}) {
  // A modifier or another key that makes no shortcut on its own says
  // something only when a held key (Glide's) is on it.
  const isComboKey =
    hoveredKey !== null &&
    !/^((Alt|Control|Meta|Shift)(Left|Right)|Fn|CapsLock)$/.test(hoveredKey);
  const value = hoveredKey ? hotkeyValue(hoveredKey, [...latched]) : null;
  const uses = entries.flatMap((entry) =>
    entry.kind === "combo"
      ? shortcutValues(entry)
          .filter((use) => parseHotkey(use, isMac)?.code === hoveredKey)
          .map((use) => ({ label: entry.label, value: use }))
      : hoveredKey && entry.value === hoveredKey
        ? [
            {
              label: t("settings-shortcuts-hold", { shortcut: entry.label }),
              value: hoveredKey,
            },
          ]
        : [],
  );
  const reserved =
    value && isComboKey && scope === "anywhere" && latched.size > 0
      ? reservedUse(value, isMac)
      : null;
  if (value && (isComboKey || uses.length > 0))
    return (
      <Line icon={infoIcon}>
        {uses.map((use) => (
          <Use key={`${use.label} ${use.value}`} {...use} />
        ))}
        {reserved ? <Reserved reserved={reserved} value={value} /> : null}
        {uses.length === 0 && !reserved ? (
          <Use label={t("settings-shortcuts-free")} value={value} />
        ) : null}
      </Line>
    );
  if (notice)
    return (
      <Line
        icon={
          notice.tone === "done" ? (
            <CircleCheck
              aria-hidden
              className="size-icon-sm shrink-0 text-success"
            />
          ) : (
            <CircleAlert
              aria-hidden
              className="size-icon-sm shrink-0 text-warning"
            />
          )
        }
      >
        <Text as="span" className="truncate" variant="caption">
          {notice.text}
        </Text>
      </Line>
    );
  return (
    <Line icon={infoIcon}>
      <Text as="span" className="truncate" variant="caption">
        {isFixedScope(scope)
          ? t("settings-shortcuts-hint-fixed")
          : selected
            ? selected.kind === "hold"
              ? t("settings-shortcuts-hint-hold", { shortcut: selected.label })
              : t("settings-shortcuts-hint-selected", {
                  shortcut: selected.label,
                })
            : t("settings-shortcuts-hint")}
      </Text>
    </Line>
  );
}
