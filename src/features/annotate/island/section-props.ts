// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Ref } from "react";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

/** What every part hanging under the island's row is handed. It takes a ref
 * so `popLayout` can lift it from the flow while it fades. */
export type SectionProps = {
  onChange: (patch: Partial<AnnotateSettings>) => void;
  settings: AnnotateSettings;
  ref?: Ref<HTMLDivElement>;
};
