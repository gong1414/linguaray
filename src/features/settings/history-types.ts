export type HistoryResult = {
  result_uuid: string;
  provider_uuid: string;
  provider_name: string;
  engine_id: string;
  elapsed_ms: number;
  outcome_tag: string;
  text: string | null;
  error_kind: string | null;
  error_message: string | null;
  corrupt: boolean;
};

export type HistoryItem = {
  session_uuid: string;
  timestamp: number;
  trigger_source: string;
  detected_language: string | null;
  target_language: string;
  is_favorite: boolean;
  source_text: string | null;
  results: HistoryResult[];
  corrupt: boolean;
};

export type HistoryPage = {
  items: HistoryItem[];
  next_cursor: string | null;
  scan_complete: boolean;
};

export type HistoryFilter = {
  query?: string | null;
  favorites_only: boolean;
};
