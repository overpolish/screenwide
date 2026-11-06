// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import type { Ref } from "react";

import { Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { SettingsPane } from "../panes/settings-pane";
import { SettingsGroup } from "../settings-group";

import { LicenceKeys } from "./licence-keys";
import { PAID_TOOLS, paidToolName } from "./paid-tools";

import type { LicencesControls } from "./use-licences";
import type { HeldKey } from "../../../bindings/HeldKey";
import type { ToolAccess } from "../../../bindings/ToolAccess";

const DAY_MS = 86_400_000;

/** What a tool's row says about whether it may be used. */
function accessNote(access: ToolAccess, trialDays: number, key?: HeldKey) {
  switch (access.kind) {
    case "licensed":
      return key
        ? t("settings-licences-licensed-key", { hint: key.hint })
        : t("settings-licences-licensed");
    case "trial":
      return t("settings-licences-trial", {
        days: Math.max(
          1,
          Math.ceil((access.endsAt * 1000 - Date.now()) / DAY_MS),
        ),
      });
    case "trialUnused":
      return t("settings-licences-trial-unused", { days: trialDays });
    case "expired":
      return t("settings-licences-expired");
  }
}

// Settings › Licences: each paid tool, whether it may be used and the key
// that unlocks it, then the bundle and the field for adding a key. A tool
// turned away opens this pane.
export function LicencesPane({
  controls,
  paneRef,
}: {
  controls: LicencesControls;
  paneRef?: Ref<HTMLElement>;
}) {
  const { buy, licences, remove } = controls;
  if (!licences) return null;
  return (
    <SettingsPane label={t("settings-section-licences")} paneRef={paneRef}>
      <SettingsGroup>
        {PAID_TOOLS.map((tool) => {
          const access =
            licences.tools.find((entry) => entry.tool === tool)?.access ?? null;
          if (!access) return null;
          const key = licences.keys.find((held) => held.tools.includes(tool));
          return (
            <ControlRow
              description={accessNote(access, licences.trialDays, key)}
              key={tool}
              title={paidToolName(tool)}
            >
              {() =>
                access.kind === "licensed" ? (
                  key ? (
                    <ConfirmActionButton
                      confirmLabel={t("settings-licences-confirm-remove", {
                        hint: key.hint,
                      })}
                      icon={<Trash2 />}
                      label={t("settings-licences-remove", { hint: key.hint })}
                      onConfirm={() => {
                        remove(key.id);
                      }}
                      variant="ghost"
                    />
                  ) : null
                ) : (
                  <Button
                    aria-label={t("settings-licences-buy-tool", {
                      tool: paidToolName(tool),
                    })}
                    onPress={() => {
                      buy({ kind: "tool", tool });
                    }}
                  >
                    {t("settings-licences-buy")}
                  </Button>
                )
              }
            </ControlRow>
          );
        })}
      </SettingsGroup>
      <SettingsGroup title={t("settings-licences-bundle")}>
        <ControlRow
          description={t("settings-licences-bundle-description")}
          title={t("settings-licences-bundle-title")}
        >
          {() => (
            <Button
              onPress={() => {
                buy({ kind: "bundle" });
              }}
            >
              {t("settings-licences-buy-bundle")}
            </Button>
          )}
        </ControlRow>
      </SettingsGroup>
      <LicenceKeys controls={controls} />
    </SettingsPane>
  );
}
