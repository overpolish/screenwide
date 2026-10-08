// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { Badge } from "../../components/base/badge/badge";
import { Checkbox } from "../../components/base/checkbox/checkbox";
import { PillGroup } from "../../components/base/pill-group/pill-group";
import { t } from "../../i18n/i18n";
import { formatBytes } from "../editor/duration";

import type { KindFilter } from "./project-list";
import type { ProjectKind, ProjectSummary } from "./types";

const kindItems = (): { id: KindFilter; label: string }[] => [
  { id: "all", label: t("project-browser-kind-all") },
  { id: "screen", label: t("project-browser-kind-screen") },
  { id: "camera", label: t("project-browser-kind-camera") },
  { id: "audio", label: t("project-browser-kind-audio") },
  { id: "screenshot", label: t("project-browser-kind-screenshot") },
];

export type ProjectListToolbarProps = {
  kind: KindFilter;
  /** Kinds among the source's projects; the others cannot be picked. */
  kindsPresent: ReadonlySet<ProjectKind>;
  /** What the list shows, before any choice. */
  listed: readonly ProjectSummary[];
  onClearSelection: () => void;
  onKindChange: (kind: KindFilter) => void;
  onSelectAll: () => void;
  /** How many listed projects can be chosen: the ones that are there. */
  selectableCount: number;
  selected: readonly ProjectSummary[];
  /** What can be done to the choice, shown in place of the kind filter
   * while there is one. */
  selectionActions: ReactNode;
  /** What can be done to everything listed, at the trailing edge after the
   * counts, such as emptying Recently Deleted. */
  listActions?: ReactNode;
};

/**
 * Above the projects: a box choosing all or none of them, then the kind
 * filter or, once anything is chosen, what can be done to the choice. At the
 * trailing edge, how many there are and the disk they take, for the choice
 * when there is one, so a clean-up shows what it will free.
 */
export function ProjectListToolbar({
  kind,
  kindsPresent,
  listActions,
  listed,
  onClearSelection,
  onKindChange,
  onSelectAll,
  selectableCount,
  selected,
  selectionActions,
}: ProjectListToolbarProps) {
  const counted = selected.length > 0 ? selected : listed;
  const bytes = counted.reduce(
    (total, { sizeBytes }) => total + (sizeBytes ?? 0),
    0,
  );
  const allSelected =
    selectableCount > 0 && selected.length === selectableCount;
  const kinds = kindItems();

  return (
    // Wraps in a narrow window rather than running off it; the counts keep
    // to the trailing edge on whichever line they land.
    <div className="gap-section pr-window-inset flex min-h-control-height flex-wrap items-center">
      <Checkbox
        aria-label={
          allSelected
            ? t("project-browser-deselect-all")
            : t("project-browser-select-all")
        }
        isDisabled={selectableCount === 0}
        isIndeterminate={selected.length > 0 && !allSelected}
        isSelected={allSelected}
        onChange={() => {
          // A partial choice clears, as Mail and Photos do: the box is a
          // way out of choosing as much as a way into choosing everything.
          if (selected.length > 0) onClearSelection();
          else onSelectAll();
        }}
      />
      {selected.length > 0 ? (
        <div className="gap-control flex items-center">{selectionActions}</div>
      ) : (
        <PillGroup
          aria-label={t("project-browser-kind")}
          disabledIds={kinds
            .filter(({ id }) => id !== "all" && !kindsPresent.has(id))
            .map(({ id }) => id)}
          display="label"
          items={kinds}
          onSelectionChange={(id) => {
            onKindChange(id as KindFilter);
          }}
          selected={kind}
        />
      )}
      <div className="gap-section ml-auto flex shrink-0 items-center">
        <div className="gap-control flex items-center">
          <Badge>
            {selected.length > 0
              ? t("project-browser-selected-count", { count: selected.length })
              : t("project-browser-project-count", { count: listed.length })}
          </Badge>
          {bytes > 0 ? <Badge>{formatBytes(bytes)}</Badge> : null}
        </div>
        {selected.length > 0 ? null : listActions}
      </div>
    </div>
  );
}
