# Plan: X (Twitter) API Two-Step Reply Publishing

## Objective
Change the X (Twitter) publisher from single-post publishing (text + URL in one payload) to a two-step reply pattern: post text-only main tweet, then reply with the URL.

## Why
1. **Cost reduction (87%)**: URL surcharge ($0.20) is avoided on the main post. Text-only post costs $0.015 + reply with URL costs ~$0.010 = ~$0.025 total vs $0.20+ for single post with URL.
2. **Algorithmic distribution**: X's recommendation engine penalizes main feed posts with outbound links. Keeping links in replies preserves organic reach.

## Files to modify
- `back/src/models/publisher/x.rs` — The `XPublisher::publish()` method

## Implementation steps

### Step 1: Modify `XPublisher::publish()` in `back/src/models/publisher/x.rs`

Current behavior: concatenates `title` and `url` into a single string and posts once.

New behavior:
1. **Post main tweet (text only)**: POST `https://api.twitter.com/2/tweets` with `{"text": "<title>"}`
2. **Parse response**: Extract `data.id` from the JSON response
3. **Wait 200ms**: Use `tokio::time::sleep(Duration::from_millis(200))`
4. **Post reply with URL**: POST `https://api.twitter.com/2/tweets` with `{"text": "<url>", "reply": {"in_reply_to_tweet_id": "<id>"}}`
5. **Return**: Return a JSON string containing both tweet IDs for logging

### Error handling
- If step 1 fails, abort and return error (no tweet was posted)
- If step 1 succeeds but step 2 fails, log the main tweet ID in the error message so the reply can be retried manually
- Both requests use OAuth 2.0 Bearer token (same as current implementation)

### No changes needed to:
- `PublisherImpl` trait signature — it already receives `title`, `description`, `url` separately
- `main.rs` — the publish loop calls `publish()` generically
- `http/publishers.rs` — the test endpoint calls `publish()` generically
- Frontend — no UI changes needed
- Database schema — no changes needed