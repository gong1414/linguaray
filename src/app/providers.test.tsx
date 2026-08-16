import { render, screen } from "@testing-library/react";
import { Button } from "@fluentui/react-components";
import { describe, expect, it } from "vitest";
import { AppProviders } from "./providers";

describe("AppProviders", () => {
  it("mounts the Fluent provider and Fluent controls", () => {
    render(
      <AppProviders>
        <Button>确定</Button>
      </AppProviders>,
    );
    expect(screen.getByRole("button", { name: "确定" }).className).toContain("fui-Button");
    expect(document.querySelector(".fui-FluentProvider")).toBeInTheDocument();
  });

  it("uses the persisted color scheme", () => {
    localStorage.setItem("linguaray.theme", "dark");
    render(<AppProviders><span>probe</span></AppProviders>);
    expect(document.querySelector("[data-color-scheme='dark']")).toBeInTheDocument();
  });
});
