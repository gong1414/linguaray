// @ts-check
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

/**
 * Architecture boundary lint (migration spec §二.8).
 *
 *  1. `@tauri-apps/*` may only be imported from src/bridge/**.
 *  2. view.tsx is pure UI: no bridge / controller / Tauri imports.
 *  3. The React tree (src/app/**) must never import Solid or the legacy
 *     self-built kit.
 * Mirrors of rules 1–2 also live as vitest gates (bridge-boundary,
 * window-permission-gate) so the boundary holds even without eslint.
 */
export default tseslint.config(
  { ignores: ["**/node_modules/**", "**/dist/**", "**/target/**", "**/.storybook-static/**", "**/coverage/**", "packages/**", "apps/**"] },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parserOptions: { ecmaVersion: "latest", sourceType: "module" },
    },
    rules: {
      // "_"-prefixed params/vars are the repo's deliberate-unused convention.
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_", caughtErrorsIgnorePattern: "^_" },
      ],
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@tauri-apps/*"],
              message: "Tauri APIs are only reachable via src/bridge/* (docs/UI-RULES.md rule 3).",
            },
          ],
        },
      ],
    },
  },
  {
    // The bridge IS the choke point — it may import @tauri-apps.
    files: ["src/bridge/**/*.{ts,tsx}"],
    rules: { "no-restricted-imports": "off" },
  },
  {
    // Legacy Solid tree is frozen (deleted in migration Phase 5) — its
    // idioms (`let ref;` bindings, expression-only focus calls) are not
    // worth churning. The React tree (src/app) stays fully strict.
    files: ["src/{App,InputPanel,OcrOverlay,Onboarding,Popup,main,theme,i18n}.tsx", "src/*-entry.tsx", "src/features/**/*.{ts,tsx}"],
    rules: {
      "no-unassigned-vars": "off",
      "@typescript-eslint/no-unused-expressions": "off",
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  {
    // Test mocks use "_"-prefixed unused params and `any` stubs by design.
    files: ["test/**/*.{ts,tsx}", "src/**/*.test.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_", caughtErrorsIgnorePattern: "^_", caughtErrors: "none" },
      ],
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  {
    // React tree hygiene: no Solid, no legacy kit, no direct bridge access
    // from view files (controllers own bridge calls).
    files: ["src/app/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            { group: ["solid-js", "solid-js/*", "lucide-solid"], message: "The React tree must not import Solid." },
            { group: ["@linguaray/ui", "@linguaray/ui/*"], message: "The React tree must not use the frozen legacy kit." },
            { group: ["@tauri-apps/*"], message: "Tauri APIs are only reachable via src/bridge/*." },
          ],
        },
      ],
    },
  },
  {
    // view.tsx is presentational: props/callbacks only.
    files: ["src/app/**/view.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            { group: ["**/bridge/*", "**/bridge/**", "**/controller", "**/controller.ts", "../../bridge/*", "../../../bridge/*"], allowTypeImports: true, message: "view.tsx is pure UI — bridge/IPC access belongs in controller.ts. (Type-only imports are allowed so views can reference controller-owned state types.)" },
          ],
        },
      ],
    },
  },
);
