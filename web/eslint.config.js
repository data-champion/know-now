import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist/"] },
  tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
);
