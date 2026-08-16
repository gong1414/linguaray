/// <reference types="vitest/globals" />
/// <reference types="@testing-library/jest-dom" />

// CSS Modules imported by React components (values are class-name maps at
// runtime under Vite; inert class strings under jsdom).
declare module "*.module.css" {
  const classes: Readonly<Record<string, string>>;
  export default classes;
}
