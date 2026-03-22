# Suggestions Endpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a free-form suggestions endpoint (MCP + Web) that dispatches a `Suggestion` event through the existing event system, plus a frontend suggestions page.

**Architecture:** New `SuggestionEvent` variant flows through the existing `EventDispatcher` to all listeners (SQLite activity + Telegram). No new storage, traits, or config. The API handler validates and trims the message (max 2000 chars). Frontend gets a simple form page at `/suggestions`.

**Tech Stack:** Rust (axum, rmcp, serde, schemars), SvelteKit 5 (runes), Tailwind CSS 4

**Spec:** `docs/superpowers/specs/2026-03-22-suggestions-endpoint-design.md`

---

### Task 1: Add SuggestionEvent to event_processing

**Files:**
- Modify: `event_processing/src/lib.rs`

- [ ] **Step 1: Add the SuggestionEvent struct**

Add after the `PaperOpenEvent` struct (around line 100):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SuggestionEvent {
    pub message: String,
}
```

- [ ] **Step 2: Add the Event::Suggestion variant**

Add to the `Event` enum after `PaperOpen(PaperOpenEvent)`:

```rust
Suggestion(SuggestionEvent),
```

- [ ] **Step 3: Add the event_type() match arm**

Add to `Event::event_type()` match block:

```rust
Event::Suggestion(_) => "suggestion",
```

- [ ] **Step 4: Update test_event_type_strings test**

Add to the `cases` vec in `test_event_type_strings`:

```rust
(Event::Suggestion(SuggestionEvent { message: "test suggestion".into() }), "suggestion"),
```

- [ ] **Step 5: Update test_event_serialization_all_variants test**

Add to the `events` vec in `test_event_serialization_all_variants`:

```rust
Event::Suggestion(SuggestionEvent { message: "improve search".into() }),
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p event_processing`
Expected: All tests pass, including the updated ones.

- [ ] **Step 7: Commit**

```bash
git add event_processing/src/lib.rs
git commit -m "feat: add SuggestionEvent to event system"
```

---

### Task 2: Create API handler

**Files:**
- Create: `api/src/suggestion.rs`
- Modify: `api/src/lib.rs`

- [ ] **Step 1: Write the test**

Create `api/src/suggestion.rs` with the test first:

```rust
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, SuggestionEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestionArgs {
    #[schemars(description = "The suggestion or feedback message (max 2000 characters)")]
    pub message: String,
}

pub async fn submit_suggestion(
    args: SuggestionArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<String, ApiError> {
    let message = args.message.trim().to_string();

    if message.is_empty() {
        return Err(ApiError::Argument(
            "message".to_string(),
            "Suggestion message cannot be empty".to_string(),
        ));
    }

    if message.len() > 2000 {
        return Err(ApiError::Argument(
            "message".to_string(),
            "Suggestion message cannot exceed 2000 characters".to_string(),
        ));
    }

    dispatcher.dispatch(
        source,
        Event::Suggestion(SuggestionEvent { message }),
    );

    Ok("Suggestion received, thank you!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_suggestion_success() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "Add year range filtering".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Suggestion received, thank you!");
    }

    #[tokio::test]
    async fn test_submit_suggestion_empty_message() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_whitespace_only() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "   \n\t  ".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_too_long() {
        let long_message = "a".repeat(2001);
        let result = submit_suggestion(
            SuggestionArgs {
                message: long_message,
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_trims_whitespace() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "  Add year range filtering  ".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Register the module**

Add to `api/src/lib.rs`:

```rust
pub mod suggestion;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p api`
Expected: All tests pass including the new suggestion tests.

- [ ] **Step 4: Commit**

```bash
git add api/src/suggestion.rs api/src/lib.rs
git commit -m "feat: add submit_suggestion API handler with validation"
```

---

### Task 3: Add MCP tool

**Files:**
- Modify: `mcp/src/server.rs`

- [ ] **Step 1: Add the import**

Add `suggestion` to the `use api::{...}` import at the top of `mcp/src/server.rs`.

- [ ] **Step 2: Add the tool method**

Add inside the `#[tool_router] impl AppServer` block:

```rust
#[tool(
    description = "Submit a suggestion or feedback message about the TDP search system. Use this to report issues, request features, or suggest improvements. The message is free-form text (max 2000 characters)."
)]
pub async fn submit_suggestion(
    &self,
    Parameters(args): Parameters<suggestion::SuggestionArgs>,
) -> Result<CallToolResult, McpError> {
    match suggestion::submit_suggestion(
        args,
        &self.state.dispatcher,
        event_processing::EventSource::Mcp,
    )
    .await
    {
        Ok(response) => Ok(CallToolResult::success(vec![Content::text(response)])),
        Err(e) => Err(McpError::internal_error(e.to_string(), None)),
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p mcp`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add submit_suggestion MCP tool"
```

---

### Task 4: Add Web route

**Files:**
- Create: `web/src/routes/suggestion.rs`
- Modify: `web/src/routes/mod.rs`

- [ ] **Step 1: Create the route handler**

Create `web/src/routes/suggestion.rs`:

```rust
use axum::extract::State;
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn submit_suggestion_handler(
    State(state): State<AppState>,
    Json(args): Json<api::suggestion::SuggestionArgs>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let result = api::suggestion::submit_suggestion(
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
```

- [ ] **Step 2: Register the route**

In `web/src/routes/mod.rs`:

Add `mod suggestion;` to the module declarations at the top.

Add the route to the `api_routes` Router chain, alongside the other `.route(...)` calls (before the `.layer(middleware::...)` call):

```rust
.route("/api/suggestion", post(suggestion::submit_suggestion_handler))
```

Note: `post` is already imported in `mod.rs`.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p web`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/routes/suggestion.rs web/src/routes/mod.rs
git commit -m "feat: add POST /api/suggestion web route"
```

---

### Task 5: Add frontend API function and types

**Files:**
- Modify: `frontend/src/lib/api.ts`

- [ ] **Step 1: Add the submitSuggestion function**

Add to `frontend/src/lib/api.ts`:

```typescript
export async function submitSuggestion(message: string, fetchFn?: FetchFn): Promise<string> {
	return fetchApi<string>('/suggestion', fetchFn, {
		method: 'POST',
		body: JSON.stringify({ message })
	});
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/lib/api.ts
git commit -m "feat: add submitSuggestion API client function"
```

---

### Task 6: Create frontend suggestions page

**Files:**
- Create: `frontend/src/routes/suggestions/+page.svelte`

- [ ] **Step 1: Create the suggestions page**

Create `frontend/src/routes/suggestions/+page.svelte`:

```svelte
<script lang="ts">
	import { submitSuggestion } from '$lib/api';

	let message = $state('');
	let status: 'idle' | 'submitting' | 'success' | 'error' = $state('idle');
	let errorMessage = $state('');

	const maxLength = 2000;
	let charCount = $derived(message.length);
	let canSubmit = $derived(message.trim().length > 0 && message.length <= maxLength && status !== 'submitting');

	async function handleSubmit() {
		if (!canSubmit) return;

		status = 'submitting';
		errorMessage = '';

		try {
			await submitSuggestion(message);
			status = 'success';
			message = '';
		} catch (e) {
			status = 'error';
			errorMessage = e instanceof Error ? e.message : 'Something went wrong';
		}
	}
</script>

<div class="max-w-2xl mx-auto px-4 py-8 sm:py-12">
	<h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">Suggestions</h1>
	<p class="text-gray-600 dark:text-gray-400 mb-6">
		Have an idea to improve TDP Browser? Found a bug? Let us know!
	</p>

	{#if status === 'success'}
		<div class="bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg p-4 mb-6">
			<p class="text-green-800 dark:text-green-300">Thank you for your suggestion!</p>
		</div>
		<button
			onclick={() => { status = 'idle'; }}
			class="text-blue-600 dark:text-blue-400 hover:underline"
		>
			Submit another
		</button>
	{:else}
		<form onsubmit={e => { e.preventDefault(); handleSubmit(); }}>
			<textarea
				bind:value={message}
				placeholder="Your suggestion or feedback..."
				rows="5"
				maxlength={maxLength}
				class="w-full px-4 py-3 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y"
			></textarea>

			<div class="flex items-center justify-between mt-2">
				<span class="text-sm text-gray-500 dark:text-gray-400">
					{charCount}/{maxLength}
				</span>

				<button
					type="submit"
					disabled={!canSubmit}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					{#if status === 'submitting'}
						Sending...
					{:else}
						Send Suggestion
					{/if}
				</button>
			</div>
		</form>

		{#if status === 'error'}
			<div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4 mt-4">
				<p class="text-red-800 dark:text-red-300">{errorMessage}</p>
			</div>
		{/if}
	{/if}
</div>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/routes/suggestions/+page.svelte
git commit -m "feat: add suggestions page to frontend"
```

---

### Task 7: Add navigation link to suggestions page

**Files:**
- Modify: `frontend/src/lib/components/Navbar.svelte`

- [ ] **Step 1: Add link to navbar**

In `Navbar.svelte`, add a suggestions link next to the ThemeToggle. Replace the Theme Toggle div:

```svelte
<!-- Right side: Suggestions link + Theme Toggle -->
<div class="flex-shrink-0 flex items-center space-x-4">
	<a href="/suggestions" class="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors">
		Suggestions
	</a>
	<ThemeToggle />
</div>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/lib/components/Navbar.svelte
git commit -m "feat: add suggestions link to navbar"
```

---

### Task 8: Update CLAUDE.md documentation

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update the "Adding a New Tool / Endpoint" section**

No change needed to the pattern section — it still applies. But add `submit_suggestion` to any tool/endpoint listing if one exists, and ensure the event list is up to date.

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with suggestion endpoint"
```

---

### Task 9: Run full test suite and verify

- [ ] **Step 1: Run all backend tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Build frontend**

Run: `cd frontend && npm run build`
Expected: Build succeeds without errors.

- [ ] **Step 3: Verify frontend type checking**

Run: `cd frontend && npm run check`
Expected: No type errors.
