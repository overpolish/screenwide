// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn, elementFocusVisible } from "../../../../lib/styling";
import { fieldVariants } from "../../../../primitives/field/field-variants";

// The recorder reads as a field pressed to set, the raised field a select
// is drawn in. A shortcut shown elsewhere to be pressed through to its field,
// such as a moment's, takes the same look.
export const hotkeyTriggerClassName = cn(
  fieldVariants({ material: "raised" }).field(),
  // Centred: with no chevron to balance it, a value at the leading edge
  // would leave the rest of the field empty.
  "w-40 cursor-default justify-center gap-inline px-inset text-body",
  "data-[hovered]:control-raised-hover data-[pressed]:control-raised-pressed data-[disabled]:opacity-40",
  elementFocusVisible,
);
