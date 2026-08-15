import { render, screen } from "@testing-library/react";
import { Button, MantineThemeProvider, useMantineTheme } from "@mantine/core";
import { describe, expect, it } from "vitest";
import { AppProviders } from "./providers";
import { linguaTheme } from "../ui/theme";

describe("AppProviders", () => {
  it("mounts Mantine with the LinguaRay theme", () => {
    let seen!: { primaryColor: string; defaultRadius: string };
    function Probe() {
      const theme = useMantineTheme();
      seen = { primaryColor: theme.primaryColor, defaultRadius: theme.defaultRadius as string };
      return <Button>确定</Button>;
    }
    render(
      <AppProviders>
        <Probe />
      </AppProviders>,
    );
    expect(seen.primaryColor).toBe("brand");
    expect(seen.defaultRadius).toBe("md");
    // Mantine Button static class applied → stylesheet pipeline live.
    expect(screen.getByRole("button", { name: "确定" }).className).toContain("mantine-Button-root");
  });

  it("theme maps the frozen brand tokens (light primary indigo-600)", () => {
    expect(linguaTheme.colors?.brand?.[6]).toBe("#4F46E5");
    expect(linguaTheme.colors?.brand?.[4]).toBe("#818CF8");
  });
});

// Keep the import graph honest: theme object is directly usable for
// storybook-style isolated mounting too.
void MantineThemeProvider;
