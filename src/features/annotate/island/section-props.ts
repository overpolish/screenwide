// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

/** What every part hanging under the island's row is handed. */
export type SectionProps = {
  onChange: (patch: Partial<AnnotateSettings>) => void;
  settings: AnnotateSettings;
};
