import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import { Copy, AlertTriangle } from "lucide-solid";
import ResultCard, { type ResultAction } from "./ResultCard";
import { assertNoAxeViolations } from "../../test/setup";

const actions: ResultAction[] = [
  { label: "Copy", icon: <Copy size={14} /> },
];

describe("ResultCard", () => {
  it("renders success text and engine label", () => {
    const { getByText } = render(
      () => (
        <ResultCard
          engineId="deepseek"
          engineLabel="DeepSeek"
          text="Hello"
          elapsedMs={420}
          outcome="success"
        />
      ),
    );
    expect(getByText("Hello")).toBeTruthy();
    expect(getByText("DeepSeek")).toBeTruthy();
    expect(getByText("420 ms")).toBeTruthy();
  });

  it("renders failure error text in destructive color and no actions", () => {
    const { queryByLabelText, getByText } = render(
      () => (
        <ResultCard
          engineId="openai"
          engineLabel="OpenAI"
          outcome="failure"
          errorText="Network error"
          actions={actions}
        />
      ),
    );
    expect(getByText("Network error")).toBeTruthy();
    // Failed card renders no action buttons.
    expect(queryByLabelText("Copy")).toBeNull();
  });

  it("renders action buttons as accessible icon buttons", () => {
    const { getByLabelText } = render(
      () => (
        <ResultCard
          engineId="deepseek"
          engineLabel="DeepSeek"
          text="Hello"
          outcome="success"
          actions={[
            { label: "Copy", icon: <Copy size={14} />, active: true },
          ]}
        />
      ),
    );
    const copy = getByLabelText("Copy");
    expect(copy.tagName).toBe("BUTTON");
    // active → aria-pressed
    expect(copy.getAttribute("aria-pressed")).toBe("true");
  });

  it("has no axe violations", async () => {
    render(() => (
      <ResultCard
        engineId="deepseek"
        engineLabel="DeepSeek"
        text="Hello"
        outcome="success"
        actions={[
          { label: "Copy", icon: <Copy size={14} /> },
          { label: "Error", icon: <AlertTriangle size={14} /> },
        ]}
      />
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
