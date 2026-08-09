import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import SidebarItem from "./SidebarItem";
import { assertNoAxeViolations } from "../../test/setup";

describe("SidebarItem", () => {
  it("renders label and icon", () => {
    const { getByText } = render(() => <SidebarItem label="Home" icon={<span data-testid="ic" />} />);
    expect(getByText("Home")).toBeInTheDocument();
  });

  it("active sets aria-current=page", () => {
    const { container } = render(() => <SidebarItem label="Home" icon={<i />} active />);
    expect(container.querySelector("[aria-current='page']")).not.toBeNull();
  });

  it("badge renders", () => {
    const { getByText } = render(() => <SidebarItem label="Home" icon={<i />} badge="3" />);
    expect(getByText("3")).toBeInTheDocument();
  });

  it("onClick fires on click", () => {
    const onClick = vi.fn();
    const { getByRole } = render(() => <SidebarItem label="Home" icon={<i />} onClick={onClick} />);
    fireEvent.click(getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("disabled sets aria-disabled and stays focusable (NOT native disabled)", () => {
    // rev-9: disabled items announce via aria-disabled but remain focusable so
    // keyboard + SR users can discover the placeholder hint. Native disabled
    // would drop them from the tab order.
    const { getByRole } = render(() => <SidebarItem label="Home" icon={<i />} disabled />);
    const btn = getByRole("button") as HTMLButtonElement;
    expect(btn.getAttribute("aria-disabled")).toBe("true");
    expect(btn.hasAttribute("disabled")).toBe(false);
    expect(btn.getAttribute("tabindex")).not.toBe("-1");
  });

  it("disabled item does NOT fire onClick", () => {
    const onClick = vi.fn();
    const { getByRole } = render(() => <SidebarItem label="Home" icon={<i />} disabled onClick={onClick} />);
    fireEvent.click(getByRole("button"));
    expect(onClick).not.toHaveBeenCalled();
  });

  it("active applies selected token class (accent bar styling)", () => {
    const { container } = render(() => <SidebarItem label="Home" icon={<i />} active />);
    expect(container.querySelector(".sidebar-item--active")).not.toBeNull();
  });

  // Enter/Space 键盘激活由原生 <button> 自动支持，不在 vitest 中人工模拟。
  // 真实键盘交互通过 Playwright e2e 验证（见 sidebar-keyboard.visual.spec.ts）。

  it("no axe violations", async () => {
    render(() => <SidebarItem label="Settings" icon={<i />} />);
    await assertNoAxeViolations();
  });
});
