CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  hmac TEXT NOT NULL,
  contents TEXT NOT NULL,
  metadata TEXT NOT NULL,
  public BOOLEAN NOT NULL DEFAULT TRUE,
  research BOOLEAN NOT NULL DEFAULT TRUE,
  deleted BOOLEAN NOT NULL DEFAULT FALSE,
  user_id TEXT NOT NULL
)
