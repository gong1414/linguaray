import { render, screen } from "@testing-library/react";
import { Button } from "antd";
import { describe, expect, it } from "vitest";
import { AppProviders } from "./providers";

describe("AppProviders", () => {
  it("mounts the Ant Design X provider and Ant controls", () => {
    render(
      <AppProviders>
        <Button>确定</Button>
      </AppProviders>,
    );
    expect(screen.getByRole("button", { name: "确定" }).className).toContain("ant-btn");
    expect(document.querySelector(".ant-app")).toBeInTheDocument();
  });

  it("uses the persisted color scheme", () => {
    localStorage.setItem("linguaray.theme", "dark");
    render(<AppProviders><span>probe</span></AppProviders>);
    expect(document.querySelector("[data-color-scheme='dark']")).toBeInTheDocument();
  });
});
