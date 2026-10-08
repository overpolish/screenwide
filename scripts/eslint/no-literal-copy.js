// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Text a person reads belongs in `locales/`, reached through `t`. This flags
// words written straight into the UI: JSX text, strings placed as JSX
// children, and string literals given to a prop or object key that carries
// copy (`label`, `title`, `aria-label`, any `…Label`, and the like). Strings
// elsewhere, such as class names, ids and variants, pass.

const COPY_NAMES = new Set([
  "alt",
  "aria-description",
  "aria-label",
  "aria-valuetext",
  "description",
  "label",
  "message",
  "placeholder",
  "section",
  "title",
  "tooltip",
]);

const isCopyName = (name) => COPY_NAMES.has(name) || /Label$/u.test(name);

// A Tailwind class list, such as a `tv` slot named `label` or `title`: every
// word carries a hyphen, colon or bracket, and none is capitalised.
const isClassList = (text) =>
  !/\p{Lu}/u.test(text) &&
  text
    .trim()
    .split(/\s+/u)
    .every((word) => /[-:[]/u.test(word));

const hasWords = (text) => /\p{L}/u.test(text) && !isClassList(text);

/** The words in a literal, a template's fixed text, or either side of a
 * conditional, or null when there are none. */
function wordsIn(node) {
  if (!node) return null;
  if (node.type === "Literal" && typeof node.value === "string") {
    return hasWords(node.value) ? node.value : null;
  }
  if (node.type === "TemplateLiteral") {
    const text = node.quasis.map(({ value }) => value.cooked ?? "").join("…");
    return hasWords(text) ? text : null;
  }
  if (node.type === "ConditionalExpression") {
    return wordsIn(node.consequent) ?? wordsIn(node.alternate);
  }
  if (node.type === "LogicalExpression") return wordsIn(node.right);
  if (node.type === "JSXExpressionContainer") return wordsIn(node.expression);
  return null;
}

const keyName = (key) =>
  key.type === "Identifier"
    ? key.name
    : key.type === "Literal"
      ? key.value
      : null;

export const noLiteralCopy = {
  create(context) {
    const report = (node, text) => {
      context.report({
        data: { text: text.trim().slice(0, 40) },
        messageId: "literal",
        node,
      });
    };
    return {
      JSXAttribute(node) {
        const name = node.name.type === "JSXIdentifier" ? node.name.name : null;
        if (!name || !isCopyName(name)) return;
        const text = wordsIn(node.value);
        if (text) report(node, text);
      },
      JSXExpressionContainer(node) {
        if (node.parent.type === "JSXAttribute") return;
        const text = wordsIn(node.expression);
        if (text) report(node, text);
      },
      JSXText(node) {
        if (hasWords(node.value)) report(node, node.value);
      },
      Property(node) {
        const name = node.computed ? null : keyName(node.key);
        if (typeof name !== "string" || !isCopyName(name)) return;
        const text = wordsIn(node.value);
        if (text) report(node, text);
      },
    };
  },
  meta: {
    docs: {
      description: "Keep user-visible text in locales/ and read it with t()",
    },
    messages: {
      literal:
        'User-visible text "{{text}}" is written in the code; add a message to locales/en-US and use t().',
    },
    schema: [],
    type: "suggestion",
  },
};
