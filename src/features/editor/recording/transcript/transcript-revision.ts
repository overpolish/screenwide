// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

/**
 * Which transcript the editor holds of the recording open, published by the
 * Transcript tool, so what is made from its words elsewhere in the window -
 * the Captions lane - knows when to make itself again: when the transcript
 * is read, or made afresh.
 */
export const useTranscriptRevision = create<{
  artifactId: number | null;
  isReady: boolean;
  revision: number;
}>()(() => ({ artifactId: null, isReady: false, revision: 0 }));
