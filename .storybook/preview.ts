// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withThemeByClassName } from "@storybook/addon-themes";
import { mockIPC } from "@tauri-apps/api/mocks";
import { themes } from "storybook/theming";

import type { Decorator, Preview } from "@storybook/react-vite";

import "../src/index.css";
import "./styles.css";

const isNativePreview =
  new URLSearchParams(window.location.search).get("screenwide-native") === "1";

if (isNativePreview) {
  document.documentElement.classList.add("screenwide-native-preview");
  const systemDarkMode = window.matchMedia("(prefers-color-scheme: dark)");
  const synchronizeNativeTheme = ({
    matches,
  }: MediaQueryListEvent | MediaQueryList) => {
    document.documentElement.classList.toggle("dark", matches);
    document.documentElement.classList.toggle("light", !matches);
  };

  synchronizeNativeTheme(systemDarkMode);
  systemDarkMode.addEventListener("change", synchronizeNativeTheme);
}

// Stub the Tauri runtime so stories mounting Tauri-touching components render
// outside the desktop app. This installs `window.__TAURI_INTERNALS__` with
// `invoke`, `transformCallback`, `unregisterCallback`, `runCallback`, and the
// event-plugin handler. Without it, e.g. `new Channel()` throws because
// `transformCallback` is undefined. Run as a module-load side effect so the
// runtime exists before the first story's effects fire. Commands need not
// return real data - components render for layout, not live pixels - so unknown
// commands resolve to `null` and never throw.
if (!isNativePreview) {
  mockIPC(() => null, { shouldMockEvents: true });
}

// Initialize React Aria's focus tracking before Storybook's test loader wraps
// HTMLElement.prototype.focus. Lazy initialization after that wrapper is unsafe.
const [reactAria, storybookComponents] = await Promise.all([
  import("react-aria"),
  import("storybook/internal/components"),
]);

Object.freeze([reactAria.useOverlay, storybookComponents.Button]);

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    docs: {
      theme: themes.dark,
    },
    options: {
      storySort: {
        method: "alphabetical",
      },
    },
  },
  tags: ["autodocs"],
};

export const decorators = (
  isNativePreview
    ? []
    : [
        withThemeByClassName({
          defaultTheme: "dark",
          parentSelector: "html",
          themes: {
            dark: "dark",
            light: "light",
          },
        }),
      ]
) as Decorator[];

export default preview;
