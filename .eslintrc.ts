import js from "@eslint/js";
import tsParser from "@typescript-eslint/parser";
import type { Linter } from "eslint";
import eslintConfigPrettier from "eslint-config-prettier/flat";
import prettier from "eslint-plugin-prettier/recommended";
import sonarjs from "eslint-plugin-sonarjs";
import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";

const config: (Linter.Config | Linter.BaseConfig)[] = defineConfig(
  js.configs.recommended,
  tseslint.configs.recommended,
  sonarjs.configs!.recommended as Linter.Config,
  prettier,
  eslintConfigPrettier,
  {
    ignores: [
      "node_modules/",
      "target/",
      "types/deno.d.ts",
      "./.eslintrc.ts",
      "./cli-eslintrc.js",
    ],
  },
  {
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        tsconfigRootDir: __dirname,
        ecmaVersion: "latest",
        project: "./tsconfig.eslint.json",
        sourceType: "module",
      },
      globals: {
        Deno: "readonly",
        TextDecoder: "readonly",
        console: "readonly",
      },
    },
    rules: {
      "arrow-body-style": ["error", "always"],
      "prettier/prettier": [
        "error",
        {
          endOfLine: "auto",
        },
      ],
      "sonarjs/unused-import": "off",
      "sonarjs/no-nested-functions": "warn",
      "sonarjs/function-return-type": "warn",
      "sonarjs/no-nested-conditional": "warn",
      "sonarjs/cognitive-complexity": "warn",
      "sonarjs/no-commented-code": "warn",
      "sonarjs/no-try-promise": "off",
      "max-params": ["error", 1],
      "@typescript-eslint/no-inferrable-types": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/ban-ts-comment": "off",
      "no-invalid-this": "error",
      "no-console": "warn",
      "no-nested-ternary": "off",
    },
  },
  {
    files: ["./cli-eslintrc.js"],
    rules: {
      "@typescript-eslint/no-require-imports": "off",
    },
  },
  {
    files: ["**/*.test.ts"],
    rules: {
      "sonarjs/no-empty-test-file": "off",
    },
  },
);

/** Can't be changed to export default due to TypeScript & ESLint limitations */
module.exports = config;
