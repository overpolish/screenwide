// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { Text } from "../../../primitives/text/text";
import { dateReadingLabel } from "../search/date-reading";
import { SearchResultRow } from "../search/search-results";

import { ProjectGrid } from "./project-grid";
import { groupByDate } from "./project-list";

import type { ProjectReading } from "../activity/project-reading";
import type { ProjectSearch } from "../search/project-search";
import type { ProjectSummary } from "../types";

/**
 * What the list shows inside its scroll area: a search's finds, project by
 * project, under the date it was read as if any; for a date alone, the
 * projects recorded then as the list shows them, under that date; in a location the cards in the date groups of when they were
 * recorded; anywhere else (Recent, most recently opened first, and
 * Recently Deleted, in the order things were deleted in) the cards in the
 * order given, ungrouped.
 */
export function ProjectListResults({
  card,
  isDated,
  onOpen,
  projects,
  readings,
  search,
  today,
}: {
  card: (
    project: ProjectSummary,
    shown?: { hits?: number[]; previewAt?: number | null },
  ) => ReactNode;
  /** Grouped by when they were recorded, rather than in the order given. */
  isDated: boolean;
  /** Absent where projects cannot be opened, as in Recently Deleted. */
  onOpen: ((file: string) => void) | undefined;
  /** The source's projects of the kind chosen. */
  projects: readonly ProjectSummary[];
  readings: Readonly<Record<string, ProjectReading | null>>;
  /** What a search found, or null while nothing is searched for. */
  search: ProjectSearch | null;
  /** When the list was opened, which its date groups are counted from, or
   * the search made, which its date is said from. */
  today: Date;
}) {
  const grid = (shown: readonly ProjectSummary[]) => (
    <ProjectGrid
      groups={
        isDated
          ? groupByDate(shown, today)
          : [{ label: null, projects: [...shown] }]
      }
    >
      {(project) => card(project)}
    </ProjectGrid>
  );
  const dateHeading = search?.date ? (
    <Text as="h2" className="ps-0.5" role="status" variant="heading">
      {dateReadingLabel(search.date, today)}
    </Text>
  ) : null;
  if (search?.results === null)
    return (
      <div className="flex flex-col gap-group pt-1">
        {dateHeading}
        {grid(search.projects)}
      </div>
    );
  if (search) {
    const { results } = search;
    const found = t("project-browser-search-found", { count: results.length });
    return (
      // Room at the foot for the island, so it never covers the last row.
      <div className="flex flex-col gap-group pe-inset pt-1 pb-24">
        {dateHeading ? (
          <div className="flex flex-col gap-inline">
            {dateHeading}
            <Text className="ps-0.5 text-fg-secondary" variant="caption">
              {found}
            </Text>
          </div>
        ) : (
          <Text as="h2" className="ps-0.5" role="status" variant="heading">
            {found}
          </Text>
        )}
        {results.map((result, index) => {
          const map = readings[result.project.file]?.map ?? null;
          return (
            <SearchResultRow
              card={(shown) => card(result.project, shown)}
              index={index}
              key={result.project.file}
              map={map}
              onOpen={
                onOpen
                  ? () => {
                      onOpen(result.project.file);
                    }
                  : undefined
              }
              result={result}
            />
          );
        })}
      </div>
    );
  }
  return (
    // Room above the first row for a chosen card's outline.
    <div className="pt-1">{grid(projects)}</div>
  );
}
