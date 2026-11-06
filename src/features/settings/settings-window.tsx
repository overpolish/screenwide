// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MotionConfig, motion } from "motion/react";
import { type ReactNode, useCallback, useMemo, useState } from "react";

import { SidebarNav } from "../../composites/sidebar-nav/sidebar-nav";
import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { Alert } from "../../primitives/alert/alert";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

import { useLicencesOpen } from "./licences/use-licences-open";
import { SettingsSearchField } from "./search/settings-search-field";
import { SettingsSearchResults } from "./search/settings-search-results";
import { useSettingsSearch } from "./search/use-settings-search";
import { useSettingsApi } from "./settings-api-context";
import { clearIn } from "./settings-motion";
import { SettingsPanes } from "./settings-panes";
import { sectionTitle, type SettingsSection } from "./settings-sections";
import { settingsSidebarItems } from "./settings-sidebar-items";
import { useSettingsWindowState } from "./use-settings-window-state";

import type { TranscriptionModel } from "../transcription/types";

const NO_MODELS: readonly TranscriptionModel[] = [];

export function SettingsWindow({
  initialQuery,
  initialSection = "general",
  updateSetting,
}: {
  initialQuery?: string;
  initialSection?: SettingsSection;
  updateSetting?: ReactNode;
}) {
  const { hideSettings, minimize } = useSettingsApi();
  const [section, setSection] = useState<SettingsSection>(initialSection);
  const state = useSettingsWindowState();
  const kinds = state.moments.settings?.kinds;
  const moments = useMemo(
    () => (kinds ?? []).map((kind) => kind.name),
    [kinds],
  );
  const search = useSettingsSearch({
    initialQuery,
    models: state.transcription.state?.models ?? NO_MODELS,
    momentNames: moments,
    setSection,
  });
  const openLicences = useCallback(() => {
    search.setQuery("");
    setSection("licences");
  }, [search]);
  useLicencesOpen(openLicences);

  return (
    // Settings' motion follows the system's reduced motion: fades stay,
    // movement goes.
    <MotionConfig reducedMotion="user">
      <WindowShell
        header={
          <WindowHeader
            onClose={() => void hideSettings()}
            onMinimize={() => void minimize()}
            title={
              search.isActive
                ? t("settings-search-results")
                : sectionTitle(section)
            }
          />
        }
      >
        {/* Only the sidebar keeps the window inset; the content column runs to
          the window's end and bottom edges so its scroll fade lands there,
          and carries the inset on the scrolled content instead. */}
        <div className="flex min-h-0 grow gap-group ps-inset">
          <div className="flex shrink-0 flex-col gap-separate pb-inset">
            <SettingsSearchField search={search} />
            <SidebarNav
              aria-label={t("settings-sections")}
              isExpandable={false}
              isExpanded
              items={settingsSidebarItems()}
              onSelectionChange={(id) => {
                search.setQuery("");
                setSection(id as SettingsSection);
              }}
              selected={section}
            />
          </div>
          <div className="flex min-h-0 min-w-0 grow flex-col gap-separate">
            {search.isActive ? (
              <ScrollArea inWindowCorner rootClassName="min-h-0 min-w-0 grow">
                <div className="flex flex-col pe-inset pb-inset">
                  <SettingsSearchResults search={search} />
                </div>
              </ScrollArea>
            ) : (
              // Each pane is its own tree, so its picture starts over when it
              // is opened again.
              <motion.div
                key={section}
                {...clearIn}
                className="flex min-h-0 min-w-0 grow flex-col"
              >
                <SettingsPanes
                  {...state}
                  onOpenShortcut={(title) => {
                    search.openRow("hotkeys", title);
                  }}
                  paneRef={search.paneRef}
                  revealing={search.revealing}
                  section={section}
                  updateSetting={updateSetting}
                />
              </motion.div>
            )}
            {state.error ? (
              <Alert className="me-inset mb-inset" color="error" role="alert">
                {state.error}
              </Alert>
            ) : null}
          </div>
        </div>
      </WindowShell>
    </MotionConfig>
  );
}
