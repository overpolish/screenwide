// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useState } from "react";

import { t } from "../../../../../i18n/i18n";
import { glide } from "../../../../../primitives/island/island-motion";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import {
  offeredColors,
  styleColor,
  styleColorLabel,
  type TextColorRole,
  withStyleColor,
} from "../../../tool-panels/text-style/text-templates";
import {
  AnnotationColorChoice,
  type AnnotationColorProps,
} from "../annotation-island-parts";

import type { TextStyle } from "../../../../../bindings/TextStyle";

/** The colours a text is drawn in, of the ones its template uses: which
 * one the grid sets, named as pills where there is more than one, and the
 * annotations' own grid under them, a colour of the user's own among them.
 * Where the template stops using the colour in hand, the type's is. The
 * names grow in above the grid as a second colour comes into use, as an
 * outline turned on does, and fold away as the last but one goes. */
export function TextColors({
  look,
  onLookChange,
  ...colorProps
}: AnnotationColorProps & {
  look: TextStyle;
  onLookChange: (patch: Partial<TextStyle>) => void;
}) {
  const roles = offeredColors(look);
  const [picked, setPicked] = useState<TextColorRole>("text");
  const role = roles.includes(picked) ? picked : "text";
  const reduce = useReducedMotion();
  return (
    <div className="flex flex-col">
      <AnimatePresence initial={false}>
        {roles.length > 1 ? (
          <motion.div
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            initial={{ height: 0, opacity: 0 }}
            key="roles"
            transition={reduce ? { duration: 0 } : glide}
          >
            {/* The gap to the grid travels with the names, so nothing
             * jumps as they come and go. */}
            <PillGroup
              aria-label={t("editor-island-text-color-role")}
              className="mb-inline flex-wrap"
              items={roles.map((each) => ({
                id: each,
                label: styleColorLabel(each),
              }))}
              onSelectionChange={(id) => {
                setPicked(id as TextColorRole);
              }}
              selected={role}
            />
          </motion.div>
        ) : null}
      </AnimatePresence>
      <AnnotationColorChoice
        {...colorProps}
        color={styleColor(look, role)}
        onChange={(color) => {
          onLookChange(withStyleColor(look, role, color));
        }}
      />
    </div>
  );
}
