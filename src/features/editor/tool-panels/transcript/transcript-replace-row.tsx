// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown, ReplaceAll } from "lucide-react";
import { useState } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IconButton } from "../../../../primitives/button/icon-button";
import { TextField } from "../../../../primitives/input-fields/text-field";

import { replacementCorrections } from "./transcript-replace";
import { TranscriptMatch } from "./transcript-search";
import { TranscriptSearchField } from "./transcript-search-field";
import { TranscriptUnit } from "./transcript-units";

import type { TranscriptCorrection } from "../tool-panel-transcript";

/**
 * The search row with Replace all behind a toggle, VS Code style: a chevron
 * after the search field opens a row for the replacement text and the
 * button that applies it everywhere the search found, in one undo step.
 */
export function TranscriptReplaceRow({
  isDisabled,
  matches,
  onCorrectMany,
  onLeave,
  onQueryChange,
  onStep,
  query,
  units,
}: {
  isDisabled: boolean;
  matches: readonly TranscriptMatch[];
  onCorrectMany: (corrections: TranscriptCorrection[]) => void;
  onLeave: () => void;
  onQueryChange: (query: string) => void;
  onStep: (step: -1 | 1) => void;
  query: string;
  units: readonly TranscriptUnit[];
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [replacement, setReplacement] = useState("");

  return (
    <div className="flex flex-col gap-control">
      <div className="flex items-center gap-control">
        <TranscriptSearchField
          isDisabled={isDisabled}
          onLeave={onLeave}
          onQueryChange={onQueryChange}
          onStep={onStep}
          query={query}
        />
        <IconButton
          aria-expanded={isOpen}
          aria-label={t("editor-panels-transcript-replace-toggle")}
          isDisabled={isDisabled}
          onPress={() => {
            setIsOpen((open) => !open);
          }}
        >
          <ChevronDown
            className={
              isOpen
                ? "rotate-180 transition-transform"
                : "transition-transform"
            }
          />
        </IconButton>
      </div>
      {isOpen ? (
        <div className="flex items-center gap-control">
          <TextField
            aria-label={t("editor-panels-transcript-replace-with")}
            className="min-w-0 flex-1"
            icon={<ReplaceAll />}
            isDisabled={isDisabled}
            onChange={setReplacement}
            placeholder={t("editor-panels-transcript-replace-with")}
            value={replacement}
          />
          <Button
            isDisabled={isDisabled || matches.length === 0}
            onPress={() => {
              const corrections = replacementCorrections(
                units,
                matches,
                replacement,
              );
              if (corrections.length > 0) onCorrectMany(corrections);
            }}
          >
            {t("editor-panels-transcript-replace-all")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
