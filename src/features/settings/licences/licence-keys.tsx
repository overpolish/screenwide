// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useState } from "react";
import { useListFormatter } from "react-aria";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { TextField } from "../../../primitives/input-fields/text-field";
import { SettingsGroup } from "../settings-group";

import { paidToolName } from "./paid-tools";

import type { LicencesControls } from "./use-licences";
import type { AddKeyOutcome } from "../../../bindings/AddKeyOutcome";

/** What adding a key came to, in words, with the tools it unlocked. */
function useOutcomeNote(outcome: AddKeyOutcome | null) {
  const list = useListFormatter({ type: "conjunction" });
  if (!outcome) return null;
  switch (outcome.kind) {
    case "added":
      return t("settings-licences-key-added", {
        tools: list.format(outcome.tools.map(paidToolName)),
      });
    case "invalidKey":
      return t("settings-licences-key-invalid");
    case "disabled":
      return t("settings-licences-key-disabled");
    case "unavailable":
      return t("settings-licences-key-unavailable");
  }
}

// The field for adding a key from a receipt. The keys held show on their
// tools' rows above, where each can be removed.
export function LicenceKeys({ controls }: { controls: LicencesControls }) {
  const { add, added, clearAdded, isAdding } = controls;
  const [key, setKey] = useState("");
  const note = useOutcomeNote(added);
  // A key that went in leaves the field ready for the next.
  const [wasAdded, setWasAdded] = useState(added);
  if (added !== wasAdded) {
    setWasAdded(added);
    if (added?.kind === "added") setKey("");
  }
  return (
    <SettingsGroup
      description={t("settings-licences-keys-description")}
      footnote={<span role="status">{note}</span>}
      title={t("settings-licences-keys")}
    >
      <ControlRow title={t("settings-licences-add-key")}>
        {(controlProps) => (
          <form
            className="flex items-center gap-separate"
            onSubmit={(event) => {
              event.preventDefault();
              if (key.trim()) add(key.trim());
            }}
          >
            <TextField
              {...controlProps}
              className="w-56"
              isDisabled={isAdding}
              onChange={(value) => {
                setKey(value);
                if (added) clearAdded();
              }}
              placeholder={t("settings-licences-key-placeholder")}
              value={key}
            />
            <Button isDisabled={isAdding || !key.trim()} type="submit">
              {t("settings-licences-add")}
            </Button>
          </form>
        )}
      </ControlRow>
    </SettingsGroup>
  );
}
