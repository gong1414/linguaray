import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import TextArea from "./TextArea";

describe("TextArea", () => {
  it("renders the provided value", () => {
    const { getByRole } = render(
      () => <TextArea value="hello world" ariaLabel="Input" />,
    );
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    expect(ta).toBeInstanceOf(HTMLTextAreaElement);
    expect(ta.value).toBe("hello world");
  });

  it("disables the textarea when disabled is true", () => {
    const { getByRole } = render(
      () => <TextArea value="x" disabled ariaLabel="Input" />,
    );
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    expect(ta.disabled).toBe(true);
  });

  it("renders a visible label and placeholder", () => {
    const { getByLabelText, getByPlaceholderText } = render(
      () => <TextArea label="Source text" placeholder="Type something" />,
    );
    const ta = getByLabelText("Source text");
    expect(ta).toBeInstanceOf(HTMLTextAreaElement);
    expect(getByPlaceholderText("Type something")).toBeTruthy();
  });
});
