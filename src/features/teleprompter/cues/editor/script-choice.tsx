// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { ScrollText } from "lucide-react";
import { useMemo, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { SearchField } from "../../../../primitives/input-fields/search-field";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../../primitives/text/text";

import type { ProjectSummary } from "../../../project-browser/types";

/** The first line of a script, what it is about at a glance. */
const firstLine = (opening: string | null) =>
  opening?.split("\n").find((line) => line.trim() !== "") ?? "";

/** The scripts a cue can be added to, found by name, each with its first
 * line. */
export function ScriptChoice({
  hasFailed,
  isAdding,
  onChoose,
  scripts,
}: {
  hasFailed: boolean;
  isAdding: boolean;
  onChoose: (script: ProjectSummary) => void;
  /** Newest first; null until listed. */
  scripts: readonly ProjectSummary[] | null;
}) {
  const [query, setQuery] = useState("");
  const shown = useMemo(() => {
    const words = query.trim().toLocaleLowerCase();
    return (scripts ?? []).filter((script) =>
      script.title.toLocaleLowerCase().includes(words),
    );
  }, [query, scripts]);

  return (
    <div className="flex size-full flex-col gap-inline p-inset-sm">
      <SearchField
        aria-label={t("teleprompter-cue-script-search")}
        autoFocus
        onChange={setQuery}
        placeholder={t("teleprompter-cue-script-search")}
        value={query}
      />
      {hasFailed ? (
        <Alert color="error">{t("teleprompter-cue-failed")}</Alert>
      ) : null}
      <ScrollArea rootClassName="min-h-0 grow">
        {scripts && shown.length === 0 ? (
          <Text
            className="p-inset text-center text-fg-secondary"
            variant="body"
          >
            {t("teleprompter-cue-no-scripts")}
          </Text>
        ) : (
          <ListBox
            aria-label={t("teleprompter-cue-scripts")}
            className="w-full p-0"
            disabledKeys={isAdding ? shown.map((script) => script.file) : []}
            items={shown}
            onAction={(file) => {
              const script = shown.find((each) => each.file === file);
              if (script) onChoose(script);
            }}
          >
            {(script) => (
              <ListBoxItem
                className="h-auto py-inset-sm"
                id={script.file}
                showsSelection={false}
                textValue={script.title}
              >
                <span className="flex items-center gap-separate">
                  <ScrollText aria-hidden className="size-icon shrink-0" />
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate" dir="auto">
                      {script.title}
                    </span>
                    <span
                      className="truncate text-caption text-fg-secondary"
                      dir="auto"
                    >
                      {firstLine(script.opening)}
                    </span>
                  </span>
                </span>
              </ListBoxItem>
            )}
          </ListBox>
        )}
      </ScrollArea>
    </div>
  );
}
