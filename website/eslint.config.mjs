import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";
import babelParser from "@babel/eslint-parser";
import reactHooks from "eslint-plugin-react-hooks";
import nextPlugin from "@next/eslint-plugin-next";

// Flat config refeita para ESLint 10 + TypeScript 7:
// - `eslint-config-next` saiu (bundla typescript-eslint, que joga exceção com
//   TS 7, e eslint-plugin-react antigo, que quebra no RuleContext do ESLint 10).
// - `eslint-plugin-react`, `eslint-plugin-jsx-a11y` e `@babel/eslint-parser@7`
//   saíram (peers travados no ESLint <= 9).
// - TypeScript é parseado pelo Babel 8 (sintaxe, sem type-info — não depende
//   da versão do `typescript`); type-safety segue coberta por `tsc --noEmit`.
// - Regras Next vêm do plugin standalone `@next/eslint-plugin-next`.
const tsParserOptions = {
  requireConfigFile: false,
  babelOptions: { presets: ["@babel/preset-typescript"] },
};

// Babel 8 detecta JSX pela extensão do arquivo (.tsx/.jsx); as opções
// antigas isTSX/allExtensions foram removidas.
const tsxParserOptions = {
  requireConfigFile: false,
  babelOptions: { presets: ["@babel/preset-typescript", "@babel/preset-react"] },
};

export default defineConfig([
  globalIgnores([".next/**", "out/**", "build/**", "next-env.d.ts", "node_modules/**"]),
  {
    files: ["**/*.{js,mjs,cjs}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    files: ["**/*.ts", "**/*.mts", "**/*.cts"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.browser, ...globals.node },
      parser: babelParser,
      parserOptions: tsParserOptions,
    },
  },
  {
    files: ["**/*.tsx", "**/*.jsx"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.browser, ...globals.node },
      parser: babelParser,
      parserOptions: tsxParserOptions,
    },
    plugins: {
      "react-hooks": reactHooks,
      "@next/next": nextPlugin,
    },
    rules: {
      ...reactHooks.configs.flat["recommended-latest"].rules,
      ...nextPlugin.configs["core-web-vitals"].rules,
    },
  },
  {
    files: ["**/*.ts", "**/*.mts", "**/*.cts"],
    plugins: { "@next/next": nextPlugin },
    rules: { ...nextPlugin.configs["core-web-vitals"].rules },
  },
]);
