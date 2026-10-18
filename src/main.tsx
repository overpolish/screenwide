// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StrictMode } from "react";
import { I18nProvider } from "react-aria-components";
import { createRoot } from "react-dom/client";

import { layoutLocale, loadAppLocale } from "./i18n/i18n";
import "./index.css";
import { installInactiveWindowHoverBridge } from "./lib/inactive-window-hover";
import { hostPlatform, stampPlatform } from "./lib/platform";
import { installPointerModalityGuard } from "./lib/pointer-modality";
import { synchronizeSystemAccent } from "./lib/system-accent";
import { synchronizeSystemTheme } from "./lib/theme";
import { installWindowInteractionTransitions } from "./lib/window-interaction-transitions";
import {
  forgetFailedLoads,
  recoverFromFailedLoad,
} from "./windows/window-load-recovery";
import { loadWindow } from "./windows/window-routes";

synchronizeSystemTheme();
synchronizeSystemAccent();
installInactiveWindowHoverBridge();
installPointerModalityGuard();
installWindowInteractionTransitions();

// Screenwide provides its own right-click interactions where needed, so the
// web view's own menu never shows in the app. A development build keeps it,
// as the way into the inspector.
if (!import.meta.env.DEV)
  window.addEventListener("contextmenu", (event) => {
    event.preventDefault();
  });

stampPlatform(hostPlatform);

// The language loads before the window's code, so even text a module reads
// as it is imported comes out in it.
void loadAppLocale()
  .then(() => loadWindow(window.location.pathname))
  .then((Window) => {
    forgetFailedLoads();
    if (!Window) return;
    createRoot(document.getElementById("root") as HTMLElement).render(
      <StrictMode>
        <I18nProvider locale={layoutLocale()}>
          <Window />
        </I18nProvider>
      </StrictMode>,
    );
  })
  .catch(recoverFromFailedLoad);
