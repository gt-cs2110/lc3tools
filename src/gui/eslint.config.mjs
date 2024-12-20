import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";
import parser from "vue-eslint-parser";
import pluginImport from "eslint-plugin-import";
import pluginVue from "eslint-plugin-vue";

export default tseslint.config(
    {
        // Ignore all .vite/ files
        ignores: [".vite/"]
    },
    {
        extends: [
            eslint.configs.recommended,
            tseslint.configs.eslintRecommended,
            tseslint.configs.recommended,
            pluginImport.flatConfigs.recommended,
            pluginImport.flatConfigs.electron,
            pluginImport.flatConfigs.typescript,
            pluginVue.configs["flat/recommended"],
        ],

        // Apply these rules to these types of files
        files: ["**/*.{js,ts,tsx,vue}"],

        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.es2020,
                ...globals.node,
            },

            parser,
            ecmaVersion: 6,
            sourceType: "module",

            parserOptions: {
                parser: tseslint.parser,
            },
        },

        rules: {
            // too many false positives
            "import/no-unresolved": "off",
            // eslint is correct for this lint, but it's a small webapp who cares
            "vue/multi-word-component-names": "off",
            // typescript handles it and it's causing false positives on eslint's side
            "no-redeclare": "off",
            "@typescript-eslint/no-unused-vars": ["error", {
                // ignore any unused variables and arguments starting with _
                varsIgnorePattern: "^_",
                argsIgnorePattern: "^_",
                caughtErrors: "all"
            }],
            // any is bad, but also there's too many APIs with missing types
            // that would be too annoying to type.
            // i'll trust that y'all know what y'all're doing if you do :any
            "@typescript-eslint/no-explicit-any": "off",
        },
    }
)