param(
  [string]$PLAIN_TEXT
)

if ([string]::IsNullOrWhiteSpace($PLAIN_TEXT)) {
  exit 0
}

$encoded = [System.Uri]::EscapeDataString($PLAIN_TEXT)
Start-Process "linguaray://translate?text=$encoded"
