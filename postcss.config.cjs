module.exports = {
  /* Mantine 9 CSS Modules support (@mixin, mixins) for React-tree CSS
   * modules under src/app. Legacy plain CSS (packages/ui, old entries) uses
   * none of these features and passes through untouched. */
  plugins: {
    "postcss-preset-mantine": {},
    "postcss-simple-vars": {},
    "postcss-nesting": {},
  },
};
