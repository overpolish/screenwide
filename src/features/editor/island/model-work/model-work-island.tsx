// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Cpu } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { ModelWorkLine } from "./model-work-line";

import type { ModelJob } from "../../../../bindings/ModelJob";
import type { ModelWork } from "../../../../bindings/ModelWork";
import type { EditorIslandContent } from "../editor-island";

// The work the models do on the recording, out for as long as there is any,
// in the picture's bottom start corner, away from the corner every other
// island stands in, so it never covers what is in hand. One line for each
// tool at work, in the order it started: what it is doing and how long it
// has left on the text column, its bar, and Cancel at the line's end, which
// stops the work and turns the tool off. Lines come and go as the work does
// (`IslandParts`). Cancel loses nothing that cannot be had again, so it asks
// once.

export type ModelWorkIslandProps = {
  /** The work under way, in the order it started; the island is put away
   * while there is none. */
  jobs: ModelJob[];
  onCancel: (work: ModelWork) => void;
};

export function modelWorkIsland({
  jobs,
  onCancel,
}: ModelWorkIslandProps): EditorIslandContent | null {
  if (jobs.length === 0) return null;
  return {
    key: "model-work",
    section: (
      <IslandParts className="flex flex-col gap-inline">
        {jobs.map((job) => (
          <ModelWorkLine
            job={job}
            key={job.work}
            onCancel={() => {
              onCancel(job.work);
            }}
          />
        ))}
      </IslandParts>
    ),
    title: (
      <IslandRowTitle
        glyph={<Cpu aria-hidden className="size-icon" />}
        title={t("editor-island-model-work")}
      />
    ),
  };
}
