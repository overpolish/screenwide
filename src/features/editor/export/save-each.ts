// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { ExportedFile } from "../../../bindings/ExportedFile";

/** The file the last export wrote, written again beside it as `fileStem`. */
const saveExportAgainAs = (fileStem: string) =>
  invoke<ExportedFile>("save_export_again_as", { fileStem });

/** Each of `jobs` saved in turn, the next only once the one before is
 * written, as the backend runs one export per window at a time: a job that
 * is a copy of the one before is the file just written, written again
 * under its name; every other is made by `save`, told whether it is the
 * last one made. The last file written, or null as soon as one is
 * cancelled. */
export async function saveEach<
  Job extends { fileStem: string; isCopy?: boolean },
>(
  jobs: readonly Job[],
  save: (job: Job, isLastMade: boolean) => Promise<ExportedFile | null>,
): Promise<ExportedFile | null> {
  const made = jobs.map((job, index) => (job.isCopy ? -1 : index));
  const lastMade = Math.max(...made);
  let file: ExportedFile | null = null;
  for (const [index, job] of jobs.entries()) {
    file = job.isCopy
      ? await saveExportAgainAs(job.fileStem)
      : await save(job, index === lastMade);
    if (!file) return null;
  }
  return file;
}
