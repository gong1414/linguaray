import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup, waitFor } from "@solidjs/testing-library";
import HistoryView from "../src/features/settings/HistoryView";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string): Promise<unknown> => {
    if (cmd === "history_privacy_status") {
      return { enabled: true, retention_days: 30, record_count: 1 };
    }
    if (cmd === "history_search") {
      return {
        items: [
          {
            session_uuid: "s1",
            timestamp: 1,
            trigger_source: "input",
            detected_language: null,
            target_language: "zh",
            is_favorite: false,
            source_text: "hello",
            results: [],
            corrupt: false,
          },
        ],
        next_cursor: null,
        scan_complete: true,
      };
    }
    return null;
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

describe("HistoryView", () => {
  beforeEach(() => invokeMock.mockClear());
  afterEach(() => cleanup());

  it("shows populated history rows", async () => {
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("hello")).toBeTruthy());
  });

  it("shows empty state when history is enabled but no records", async () => {
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "history_privacy_status") {
        return { enabled: true, retention_days: 30, record_count: 0 };
      }
      if (cmd === "history_search") {
        return { items: [], next_cursor: null, scan_complete: true };
      }
      return null;
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("No history yet")).toBeTruthy());
  });
});
