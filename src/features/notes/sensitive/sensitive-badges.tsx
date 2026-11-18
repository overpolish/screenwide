// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ShieldAlert } from "lucide-react";
import { Button } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Badge } from "../../../primitives/badge/badge";

import {
  groupByKind,
  type SensitiveGroup,
  sensitiveKindLabel,
} from "./sensitive-edits";

import type { SensitiveFind } from "../../../bindings/SensitiveFind";

// Over a paragraph, what in it may be private, a badge a kind: pressed, a
// badge opens what can be done about what it names.

export function SensitiveBadges({
  finds,
  onOpen,
}: {
  finds: readonly SensitiveFind[];
  /** Opens what can be done about `group`, under `anchor`. */
  onOpen: (group: SensitiveGroup, anchor: DOMRect) => void;
}) {
  return (
    <div
      aria-label={t("notes-sensitive-found")}
      className="flex flex-wrap gap-inline"
      role="group"
    >
      {groupByKind(finds).map((group) => (
        <Button
          className={cn("group rounded-part", focusStyles, elementFocusVisible)}
          key={group.kind}
          onPress={(event) => {
            onOpen(group, event.target.getBoundingClientRect());
          }}
        >
          <Badge className="transition-colors group-data-[hovered]:bg-fill-strong group-data-[hovered]:text-fg group-data-[pressed]:bg-fill-strong">
            <ShieldAlert aria-hidden className="text-error" />
            {sensitiveKindLabel(group)}
          </Badge>
        </Button>
      ))}
    </div>
  );
}
