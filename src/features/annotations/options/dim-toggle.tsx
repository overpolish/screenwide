// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import { FocusLookGlyph } from "./focus-glyphs";
import { OptionToggle } from "./option-toggle";

/** Whether a focus dims what lies around its box, as a spotlight's shade,
 * rather than blurring it, as a camera's depth of field: dimmed it is a
 * spotlight, blurred a focus. */
export function DimToggle({
  dims,
  onChange,
}: {
  /** Dimmed rather than blurred. */
  dims: boolean;
  onChange: (dims: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <FocusLookGlyph dims={isOn} />}
      isOn={dims}
      label={t("annotate-island-focus-dim")}
      onChange={onChange}
    />
  );
}
