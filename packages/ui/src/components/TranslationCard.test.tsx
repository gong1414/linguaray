import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import TranslationCard from "./TranslationCard";
import { assertNoAxeViolations } from "../../test/setup";

const labels = { loadingLabel: "Loading…", failureText: "Translation failed", retryLabel: "Retry" };
const baseProps = {
  engineId: "google",
  engineLabel: "Google",
  labels,
};

describe("TranslationCard", () => {
  it("renders result text via ResultCard on success", () => {
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} state={{ kind: "success", text: "你好", elapsedMs: 120 }} />
    ));
    expect(getByText("你好")).toBeInTheDocument();
  });

  it("loading shows Spinner with loadingLabel, not result text", () => {
    const { container, queryByText, getByText } = render(() =>
      <TranslationCard {...baseProps} state={{ kind: "loading" }} />,
    );
    expect(container.querySelector(".lr-spinner")).not.toBeNull();
    expect(getByText("Loading…")).toBeInTheDocument();
    expect(queryByText("你好")).toBeNull();
  });

  it("failure renders error text", () => {
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} state={{ kind: "failure", errorText: "Network error" }} />
    ));
    expect(getByText("Network error")).toBeInTheDocument();
  });

  // MASTER §7 TranslationCard: on failure, labels.failureText introduces the
  // error (rendered before the error text itself).
  it("failure renders labels.failureText as introductory text", () => {
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} state={{ kind: "failure", errorText: "Network error" }} />
    ));
    expect(getByText("Translation failed")).toBeInTheDocument();
  });

  it("failure with onRetry renders retry button", () => {
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} state={{ kind: "failure", errorText: "Network error" }} onRetry={() => {}} />
    ));
    expect(getByText("Retry")).toBeInTheDocument();
  });

  it("onRetry fires", () => {
    const onRetry = vi.fn();
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} state={{ kind: "failure", errorText: "Network error" }} onRetry={onRetry} />
    ));
    fireEvent.click(getByText("Retry"));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("no axe violations", async () => {
    render(() => <TranslationCard {...baseProps} state={{ kind: "success", text: "你好", elapsedMs: 120 }} />);
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
