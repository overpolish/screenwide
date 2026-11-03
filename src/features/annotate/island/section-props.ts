// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

/** What every part hanging under the island's row is handed. */
export type SectionProps = {
  onChange: (patch: Partial<AnnotateSettings>) => void;
  settings: AnnotateSettings;
};
