// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { type Ref } from "react";

import { t } from "../../../i18n/i18n";
import { splitAtPlaceables } from "../../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../../lib/hotkey";
import { cn } from "../../../lib/styling";
import { IslandListRow } from "../../../primitives/island/island-controls";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";
import { Text } from "../../../primitives/text/text";

import {
  RULER_TOLERANCES,
  type RulerSection,
  type RulerStampKeys,
} from "./ruler-island-tools";

import type { RulerTolerance } from "../../../bindings/RulerTolerance";

/** What hangs under the row, faded across as it changes. Keyed inside
 * `AnimatePresence` by its section. */
export function RulerSectionBody({
  onTolerance,
  ref,
  section,
  stampKeys,
  tolerance,
}: {
  onTolerance: (tolerance: RulerTolerance) => void;
  section: RulerSection;
  stampKeys: RulerStampKeys | null;
  tolerance: RulerTolerance;
  ref?: Ref<HTMLDivElement>;
}) {
  return (
    <IslandCrossfade ref={ref}>
      {section === "edges" ? (
        <div className="flex flex-col">
          {RULER_TOLERANCES.map(({ id, name }) => (
            <IslandListRow
              isChosen={id === tolerance}
              key={id}
              onPress={() => {
                if (id !== tolerance) onTolerance(id);
              }}
            >
              <span className="grow">{name()}</span>
              <Check
                aria-hidden
                className={cn("size-icon", id !== tolerance && "invisible")}
              />
            </IslandListRow>
          ))}
        </div>
      ) : (
        // The note sits the small inset in from the section's edges, as the
        // circles' glyphs do from theirs.
        <Text
          className="block max-w-64 px-inset-sm text-fg-secondary"
          variant="caption"
        >
          {section === "measure" ? (
            <StampHint stampKeys={stampKeys} />
          ) : section === "radius" ? (
            t("ruler-island-radius-hint")
          ) : (
            t("ruler-island-guide-hint")
          )}
        </Text>
      )}
    </IslandCrossfade>
  );
}

/** How to stamp a measurement where the pointer is, with the two keys that
 * do it drawn as keycaps, wherever the language puts them. */
function StampHint({ stampKeys }: { stampKeys: RulerStampKeys | null }) {
  const keys = [stampKeys?.horizontal ?? null, stampKeys?.vertical ?? null];
  return splitAtPlaceables(2, ([horizontal, vertical]) =>
    t("ruler-island-measure-hint", { horizontal, vertical }),
  ).map((part, index) =>
    typeof part === "string" ? (
      part
    ) : (
      <ShortcutKeys
        key={`key-${index.toString()}`}
        value={keys[part] ?? null}
      />
    ),
  );
}

/** A binding as keycaps: one key on its own, or the keys of a combination
 * joined as `Shortcut` joins them. */
function ShortcutKeys({ value }: { value: string | null }) {
  const keys = hotkeyKeys(value);
  if (keys.length === 1) {
    return <Keyboard className="align-middle">{keys[0]}</Keyboard>;
  }
  return (
    <Shortcut className="align-middle">
      {keys.flatMap((key, index) => [
        ...(index > 0 ? (["+"] as const) : []),
        <Keyboard key={key}>{key}</Keyboard>,
      ])}
    </Shortcut>
  );
}
