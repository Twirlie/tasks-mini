# Leptos Testing Guide

A comprehensive reference for testing Leptos applications — from reactive logic to full end-to-end browser tests.

---

## Table of Contents

1. [Setup & Dependencies](#1-setup--dependencies)
2. [Testing Reactive Logic](#2-testing-reactive-logic)
3. [Testing Components via SSR](#3-testing-components-via-ssr)
4. [Testing in the Browser (WASM)](#4-testing-in-the-browser-wasm)
5. [Testing Async & Resources](#5-testing-async--resources)
6. [Testing Server Functions](#6-testing-server-functions)
7. [End-to-End Tests with Playwright](#7-end-to-end-tests-with-playwright)
8. [Structuring Your Test Suite](#8-structuring-your-test-suite)
9. [Common Pitfalls](#9-common-pitfalls)
10. [Quick Reference](#10-quick-reference)

---

## 1. Setup & Dependencies

### Cargo.toml

```toml
[dependencies]
leptos = { version = "0.7", features = ["ssr"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"
tokio = { version = "1", features = ["rt", "macros"] }  # for async tests

[features]
# SSR feature must be enabled for server-side rendering tests
ssr = ["leptos/ssr"]
```

### `.cargo/config.toml` — set default test target for WASM tests

```toml
[alias]
test-wasm = "test --target wasm32-unknown-unknown"
```

### Install wasm-pack (for browser tests)

```bash
cargo install wasm-pack
```

---

## 2. Testing Reactive Logic

Pure reactive logic — signals, memos, derived state — can be tested with plain `cargo test`. No browser, no DOM.

### Key rule: always create and dispose a runtime

Every test that uses Leptos reactivity needs a runtime. Forgetting to dispose it causes state to leak between tests.

```rust
#[cfg(test)]
mod tests {
    use leptos::*;
    use leptos::reactive_graph::owner::Owner;

    #[test]
    fn test_signal_updates() {
        // Leptos 0.7: use Owner
        let owner = Owner::new();
        owner.set();

        let (count, set_count) = signal(0);

        assert_eq!(count.get(), 0);
        set_count.set(5);
        assert_eq!(count.get(), 5);
        set_count.update(|n| *n *= 2);
        assert_eq!(count.get(), 10);

        drop(owner);
    }
}
```

> **Note:** In Leptos 0.6 and earlier, use `create_runtime()` / `runtime.dispose()`. In 0.7+, use `Owner::new()` / `drop(owner)`.

### Testing computed/memo values

```rust
#[test]
fn test_memo() {
    let owner = Owner::new();
    owner.set();

    let (count, set_count) = signal(2);
    let doubled = Memo::new(move |_| count.get() * 2);

    assert_eq!(doubled.get(), 4);
    set_count.set(10);
    assert_eq!(doubled.get(), 20);

    drop(owner);
}
```

### Testing derived signals with multiple inputs

```rust
#[test]
fn test_derived_signal() {
    let owner = Owner::new();
    owner.set();

    let (first, set_first) = signal("Hello".to_string());
    let (last, set_last) = signal("World".to_string());
    let full_name = move || format!("{} {}", first.get(), last.get());

    assert_eq!(full_name(), "Hello World");

    set_first.set("Goodbye".to_string());
    assert_eq!(full_name(), "Goodbye World");

    drop(owner);
}
```

### Testing effects

Effects are asynchronous by default in Leptos 0.7. To test that an effect ran, use a `Cell` or `Arc<Mutex<_>>` to capture results:

```rust
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_effect_runs() {
    let owner = Owner::new();
    owner.set();

    let ran = Rc::new(Cell::new(false));
    let ran_clone = ran.clone();

    let (value, set_value) = signal(0);

    Effect::new(move |_| {
        let _ = value.get(); // subscribe
        ran_clone.set(true);
    });

    // Effects don't run synchronously in 0.7 — flush first
    // Use Owner::with for synchronous effect flushing in tests
    set_value.set(1);

    assert!(ran.get());
    drop(owner);
}
```

---

## 3. Testing Components via SSR

Server-side rendering lets you test component output as HTML strings — fast, no browser required. This covers most component unit testing needs.

### Basic SSR test

```rust
#[cfg(test)]
mod tests {
    use leptos::*;
    use leptos::prelude::*;

    #[component]
    fn Greeting(name: String) -> impl IntoView {
        view! { <p class="greeting">"Hello, " {name} "!"</p> }
    }

    #[test]
    fn test_greeting_output() {
        let html = leptos::ssr::render_to_string(|| {
            view! { <Greeting name="Thomas".to_string() /> }
        });

        assert!(html.contains("Hello, Thomas!"));
        assert!(html.contains("class=\"greeting\""));
    }
}
```

### Testing conditional rendering

```rust
#[component]
fn Status(logged_in: bool) -> impl IntoView {
    view! {
        <div>
            {if logged_in {
                view! { <span class="welcome">"Welcome back!"</span> }.into_any()
            } else {
                view! { <a href="/login">"Please log in"</a> }.into_any()
            }}
        </div>
    }
}

#[test]
fn test_logged_in() {
    let html = leptos::ssr::render_to_string(|| view! { <Status logged_in=true /> });
    assert!(html.contains("Welcome back!"));
    assert!(!html.contains("Please log in"));
}

#[test]
fn test_logged_out() {
    let html = leptos::ssr::render_to_string(|| view! { <Status logged_in=false /> });
    assert!(html.contains("Please log in"));
    assert!(!html.contains("Welcome back!"));
}
```

### Testing list rendering

```rust
#[component]
fn ItemList(items: Vec<String>) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || items.clone()
                key=|item| item.clone()
                children=|item| view! { <li>{item}</li> }
            />
        </ul>
    }
}

#[test]
fn test_item_list() {
    let items = vec!["Apple".to_string(), "Banana".to_string(), "Cherry".to_string()];
    let html = leptos::ssr::render_to_string(move || {
        view! { <ItemList items=items.clone() /> }
    });

    assert!(html.contains("<li>Apple</li>"));
    assert!(html.contains("<li>Banana</li>"));
    assert!(html.contains("<li>Cherry</li>"));
}
```

### Testing props and slots

```rust
#[component]
fn Card(title: String, #[prop(optional)] subtitle: Option<String>) -> impl IntoView {
    view! {
        <div class="card">
            <h2>{title}</h2>
            {subtitle.map(|s| view! { <h3>{s}</h3> })}
        </div>
    }
}

#[test]
fn test_card_with_subtitle() {
    let html = leptos::ssr::render_to_string(|| view! {
        <Card title="My Title".to_string() subtitle="My Subtitle".to_string() />
    });
    assert!(html.contains("<h2>My Title</h2>"));
    assert!(html.contains("<h3>My Subtitle</h3>"));
}

#[test]
fn test_card_without_subtitle() {
    let html = leptos::ssr::render_to_string(|| view! {
        <Card title="My Title".to_string() />
    });
    assert!(html.contains("<h2>My Title</h2>"));
    assert!(!html.contains("<h3>"));
}
```

### Asserting HTML structure more precisely

For more structured assertions, parse with a simple string check or bring in a crate like `scraper`:

```toml
[dev-dependencies]
scraper = "0.19"
```

```rust
use scraper::{Html, Selector};

#[test]
fn test_structure_with_scraper() {
    let html_str = leptos::ssr::render_to_string(|| view! {
        <ItemList items=vec!["A".to_string(), "B".to_string()] />
    });

    let document = Html::parse_fragment(&html_str);
    let selector = Selector::parse("li").unwrap();
    let items: Vec<_> = document.select(&selector).collect();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].text().collect::<String>(), "A");
    assert_eq!(items[1].text().collect::<String>(), "B");
}
```

---

## 4. Testing in the Browser (WASM)

For tests that need real DOM interaction — events, focus, layout, client-side reactivity — use `wasm-bindgen-test`.

### Setup

```rust
// at the top of your test file
use wasm_bindgen_test::*;

// run tests in a real browser (headless)
wasm_bindgen_test_configure!(run_in_browser);
```

### Basic DOM test

```rust
use leptos::*;
use wasm_bindgen_test::*;
use web_sys::window;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_mounts_to_dom() {
    let document = window().unwrap().document().unwrap();
    let body = document.body().unwrap();

    let container = document.create_element("div").unwrap();
    body.append_child(&container).unwrap();

    leptos::mount_to(
        container.clone().unchecked_into(),
        || view! { <p id="hello">"Hello"</p> }
    );

    let el = document.get_element_by_id("hello").unwrap();
    assert_eq!(el.text_content().unwrap(), "Hello");
}
```

### Testing click events

```rust
use leptos::*;
use wasm_bindgen_test::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <div>
            <p id="count">{count}</p>
            <button id="inc" on:click=move |_| set_count.update(|n| *n += 1)>
                "Increment"
            </button>
        </div>
    }
}

#[wasm_bindgen_test]
fn test_counter_clicks() {
    let document = window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();

    leptos::mount_to(container.unchecked_into(), Counter);

    let btn = document
        .get_element_by_id("inc")
        .unwrap()
        .unchecked_into::<HtmlElement>();

    btn.click();
    btn.click();
    btn.click();

    let count_el = document.get_element_by_id("count").unwrap();
    assert_eq!(count_el.text_content().unwrap(), "3");
}
```

### Running WASM tests

```bash
# Headless Chrome
wasm-pack test --headless --chrome

# Headless Firefox
wasm-pack test --headless --firefox

# Run a specific test
wasm-pack test --headless --chrome -- --test my_test_name
```

---

## 5. Testing Async & Resources

### Testing a Resource with a mock async function

```rust
use leptos::*;
use leptos::prelude::*;

async fn fetch_user(id: u32) -> String {
    format!("User #{}", id) // replace with real fetch in production
}

#[component]
fn UserProfile(id: u32) -> impl IntoView {
    let user = Resource::new(move || id, |id| fetch_user(id));

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            <p>{move || user.get().map(|u| u)}</p>
        </Suspense>
    }
}
```

For SSR async tests, use `render_to_string_async`:

```rust
#[tokio::test]
async fn test_user_profile_ssr() {
    let html = leptos::ssr::render_to_string_async(|| {
        view! { <UserProfile id=1 /> }
    }).await;

    assert!(html.contains("User #1"));
}
```

### Injecting mock data via context

Inject mock services through Leptos context rather than calling real APIs:

```rust
#[derive(Clone)]
struct UserService {
    pub fetch: fn(u32) -> String,
}

#[component]
fn UserName(id: u32) -> impl IntoView {
    let svc = use_context::<UserService>().expect("UserService not provided");
    let name = (svc.fetch)(id);
    view! { <span>{name}</span> }
}

#[test]
fn test_user_name_with_mock() {
    let html = leptos::ssr::render_to_string(|| {
        provide_context(UserService { fetch: |id| format!("MockUser{}", id) });
        view! { <UserName id=42 /> }
    });
    assert!(html.contains("MockUser42"));
}
```

---

## 6. Testing Server Functions

Server functions are just async Rust functions — test them directly without any HTTP layer.

```rust
use leptos::server;
use leptos::ServerFnError;

#[server]
pub async fn add_todo(title: String) -> Result<u32, ServerFnError> {
    // imagine DB insert here
    Ok(42)
}

// In your integration test (not behind #[server] macro):
#[tokio::test]
async fn test_add_todo() {
    // Call the inner logic directly if you extract it:
    let result = add_todo_inner("Buy groceries".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
```

**Best practice:** extract business logic from server functions into plain async functions, then test those directly. Keep the `#[server]` wrapper thin.

```rust
// Pure logic — easily testable
pub async fn create_todo_in_db(title: String, pool: &DbPool) -> Result<u32, sqlx::Error> {
    // ...
}

// Thin server fn wrapper
#[server]
pub async fn create_todo(title: String) -> Result<u32, ServerFnError> {
    let pool = use_context::<DbPool>().ok_or(ServerFnError::ServerError("No pool".into()))?;
    create_todo_in_db(title, &pool).await.map_err(|e| ServerFnError::ServerError(e.to_string()))
}

// Test the pure logic
#[tokio::test]
async fn test_create_todo() {
    let pool = setup_test_db().await;
    let id = create_todo_in_db("Test task".to_string(), &pool).await.unwrap();
    assert!(id > 0);
}
```

---

## 7. End-to-End Tests with Playwright

For full integration tests that drive a real browser against a running server.

### Setup

```bash
npm init -y
npm install --save-dev @playwright/test
npx playwright install chromium
```

`playwright.config.ts`:
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  use: {
    baseURL: 'http://localhost:3000',
  },
  webServer: {
    command: 'cargo leptos serve',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
```

### Writing E2E tests

`e2e/counter.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';

test('counter increments on click', async ({ page }) => {
    await page.goto('/');

    const count = page.locator('#count');
    const button = page.locator('#inc');

    await expect(count).toHaveText('0');
    await button.click();
    await button.click();
    await expect(count).toHaveText('2');
});

test('form submits and shows result', async ({ page }) => {
    await page.goto('/form');

    await page.fill('input[name="username"]', 'testuser');
    await page.click('button[type="submit"]');

    await expect(page.locator('.success')).toBeVisible();
    await expect(page.locator('.success')).toContainText('testuser');
});
```

### Running E2E tests

```bash
# Run all E2E tests
npx playwright test

# Run in headed mode (see the browser)
npx playwright test --headed

# Run a specific file
npx playwright test e2e/counter.spec.ts
```

---

## 8. Structuring Your Test Suite

```
src/
├── components/
│   ├── counter.rs          # component
│   └── counter_test.rs     # SSR unit tests (mod tests inline or separate)
├── server/
│   ├── todos.rs            # server fns + business logic
│   └── todos_test.rs       # tokio::test for async logic
tests/
└── wasm/
    └── browser_tests.rs    # wasm-bindgen-test browser tests
e2e/
└── *.spec.ts               # Playwright E2E tests
```

Inline test modules are fine for small files:

```rust
// counter.rs
#[component]
pub fn Counter() -> impl IntoView { ... }

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::ssr::render_to_string;

    #[test]
    fn renders_initial_count() {
        let html = render_to_string(|| view! { <Counter /> });
        assert!(html.contains("0"));
    }
}
```

---

## 9. Common Pitfalls

| Pitfall | Fix |
|---|---|
| Forgetting `drop(owner)` / `runtime.dispose()` | Always dispose at end of test; wrap in a helper |
| Testing client-only behavior with SSR | Use `wasm-bindgen-test` for DOM events and reactive updates |
| Calling real APIs in unit tests | Inject mock services via `provide_context` |
| Effects not running synchronously | Flush effects or test via observable side effects (signal reads) |
| `features = ["ssr"]` not set | SSR rendering requires the `ssr` feature in `Cargo.toml` |
| Importing `leptos::*` vs `leptos::prelude::*` | In 0.7+ prefer `use leptos::prelude::*` |
| WASM tests running with `cargo test` | WASM tests require `wasm-pack test`, not `cargo test` |

---

## 10. Quick Reference

| What to test | Tool | Command |
|---|---|---|
| Signals, memos, derived state | `#[test]` + `Owner` | `cargo test` |
| Component HTML output | `render_to_string` | `cargo test --features ssr` |
| Async components | `render_to_string_async` | `cargo test --features ssr` |
| Server functions | `#[tokio::test]` | `cargo test` |
| DOM events, client reactivity | `wasm-bindgen-test` | `wasm-pack test --headless --chrome` |
| Full app integration | Playwright | `npx playwright test` |
