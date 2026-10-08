// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import eslintReact from "@eslint-react/eslint-plugin";
import eslint from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import eslintConfigPrettier from "eslint-config-prettier";
import importX from "eslint-plugin-import-x";
import perfectionist from "eslint-plugin-perfectionist";
import reactRefresh from "eslint-plugin-react-refresh";
import sortDestructureKeys from "eslint-plugin-sort-destructure-keys";
import globals from "globals";
import tseslint from "typescript-eslint";

import { logicalDirection } from "./scripts/eslint/logical-direction.js";
import { noLiteralCopy } from "./scripts/eslint/no-literal-copy.js";

// One object for every block that uses the house rules: ESLint refuses a
// plugin name defined twice by different objects.
const screenwide = {
  rules: {
    "logical-direction": logicalDirection,
    "no-literal-copy": noLiteralCopy,
  },
};

const frontendFiles = [
  ".storybook/**/*.{ts,tsx}",
  "src/**/*.{ts,tsx}",
  "*.config.ts",
];

export default defineConfig([
  globalIgnores([
    "dist/**",
    "node_modules/**",
    "src-tauri/**",
    "storybook-static/**",
    "src/bindings/**",
  ]),
  {
    extends: [eslint.configs.recommended],
    files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
    languageOptions: {
      globals: globals.browser,
    },
  },
  ...tseslint.configs.strictTypeChecked.map((config) => ({
    ...config,
    files: frontendFiles,
  })),
  {
    files: frontendFiles,
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/max-params": "error",
      "@typescript-eslint/member-ordering": [
        "error",
        {
          default: {
            memberTypes: ["field", "constructor", "method"],
            optionalityOrder: "required-first",
            order: "natural",
          },
        },
      ],
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    ...eslintReact.configs["recommended-type-checked"],
    files: ["src/**/*.{ts,tsx}"],
  },
  {
    ...importX.configs["flat/recommended"],
    files: frontendFiles,
    rules: {
      ...importX.configs["flat/recommended"].rules,
      "import-x/order": [
        "error",
        {
          alphabetize: {
            caseInsensitive: true,
            order: "asc",
          },
          groups: [
            "builtin",
            "external",
            "internal",
            "parent",
            "sibling",
            "index",
            "object",
            "type",
          ],
          "newlines-between": "always",
          pathGroups: [
            {
              group: "external",
              pattern: "react",
              position: "before",
            },
          ],
        },
      ],
    },
  },
  {
    ...importX.configs["flat/typescript"],
    files: frontendFiles,
  },
  {
    files: frontendFiles,
    plugins: {
      perfectionist,
      "sort-destructure-keys": sortDestructureKeys,
    },
    rules: {
      "no-restricted-exports": [
        "error",
        { restrictDefaultExports: { direct: true } },
      ],
      "perfectionist/sort-jsx-props": "error",
      "sort-destructure-keys/sort-destructure-keys": "error",
      "sort-keys": [
        "error",
        "asc",
        {
          allowLineSeparatedGroups: true,
          natural: true,
        },
      ],
    },
  },
  {
    ...reactRefresh.configs.vite,
    files: ["src/**/*.{ts,tsx}"],
  },
  {
    files: [".storybook/**/*.{ts,tsx}", "*.config.{js,ts}"],
    rules: {
      "no-restricted-exports": "off",
    },
  },
  {
    files: ["src/**/*.stories.{ts,tsx}"],
    rules: {
      "no-restricted-exports": "off",
    },
  },
  {
    // User-visible text belongs in `locales/`. Stories, tests and the
    // fixtures behind development previews hold sample data, not copy.
    files: ["src/**/*.{ts,tsx}"],
    ignores: [
      "**/*.stories.tsx",
      "**/*.test.{ts,tsx}",
      "**/*-fixtures.ts",
      "**/*-preview.ts",
      "src/storybook/**",
      // Sample tracks for the timeline stories.
      "src/features/editor/timeline/tracks/recording-track-lanes-preview.tsx",
    ],
    plugins: { screenwide },
    rules: {
      "screenwide/no-literal-copy": "error",
    },
  },
  {
    // Layouts mirror in right-to-left languages, so sides are start and end.
    files: ["src/**/*.{ts,tsx}"],
    ignores: [
      "**/*.test.{ts,tsx}",
      // Left to right in every language, so physical sides are right there.
      // Each part's root carries `dir="ltr"`. The timeline and its lanes,
      // where time runs left to right:
      "src/features/editor/timeline/**",
      "src/features/editor/recording/annotations/recording-annotation-clip-body.tsx",
      "src/features/editor/recording/annotations/recording-annotation-clip-edges.tsx",
      "src/features/editor/recording/annotations/recording-annotation-lane.tsx",
      "src/features/editor/recording/annotations/recording-annotation-pin-overlay.tsx",
      "src/features/editor/recording/scenes/recording-scene-lane.tsx",
      // The picture being edited:
      "src/features/editor/preview/**",
      "src/features/editor/recording/preview/**",
      // Screen geometry and angles:
      "src/features/glide/glide-preview.tsx",
      "src/features/recording-sources/monitor-selector.tsx",
      "src/features/editor/tool-panels/scene/scene-preset-tile.tsx",
      "src/components/base/angle-dial/**",
    ],
    plugins: { screenwide },
    rules: {
      "screenwide/logical-direction": "error",
    },
  },
  eslintConfigPrettier,
]);
