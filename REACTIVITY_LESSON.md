# Dioxus Reactivity: The Infinite Loop Bug & Solution

**Date**: 2025-10-18  
**Issue**: Infinite HTTP connection spawning after login causing UI freeze  
**Root Cause**: Misunderstanding of Dioxus reactivity tracking in `use_effect` and `use_resource`

---

## The Problem: What Went Wrong

After successful login, the app spawned **hundreds of HTTP connections per second**, freezing the UI:

```
11:49:24 [ios] DEBUG reqwest::connect: starting new connection: http://127.0.0.1:3000/
11:49:24 [ios] DEBUG reqwest::connect: starting new connection: http://127.0.0.1:3000/
11:49:24 [ios] DEBUG reqwest::connect: starting new connection: http://127.0.0.1:3000/
... (60+ more in single millisecond)
```

Initial assumption was the login component, but debugging showed login only ran **once**. The infinite loop started **after** `session.set(Some(new_session))`.

---

## Root Cause Analysis

### Problem 1: `use_effect` Spawning Infinite Background Loops

**Buggy Code** (home.rs, profile.rs, decks.rs):

```rust
use_effect(move || {
    spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let Some(s) = session.read().clone() else {  // READS session
                continue;
            };
            session.set(auth_client.read().infallible_get_active_session(&s).await);  // WRITES session
        }
    });
});
```

**Why This Broke:**
1. `use_effect` **tracks all signal reads** inside its closure
2. Reading `session` registers it as a dependency
3. Effect spawns infinite background loop
4. Loop updates `session` → triggers effect to run again
5. Effect spawns **another** infinite loop
6. Now 2 loops are running, both updating `session` → 4 loops
7. Exponential explosion: 2 → 4 → 8 → 16 → 32 → 64+ loops in milliseconds

**Fix:**

```rust
// use_future runs ONCE on mount, doesn't track dependencies
use_future(move || async move {
    let mut interval = interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let Some(s) = session.read().clone() else {
            continue;
        };
        session.set(auth_client.read().infallible_get_active_session(&s).await);
    }
});
```

---

### Problem 2: `use_resource` Reading AND Writing Same Signal

**Buggy Code** (decks.rs):

```rust
let check_session = move || async move {
    let Some(s) = session.read().clone() else {  // READS session
        return;
    };
    session.set(auth_client.read().infallible_get_active_session(&s).await);  // WRITES session
};

let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
    use_resource(move || async move {
        check_session().await;  // Updates session
        let Some(s) = session.read().clone() else {  // Reads session again
            return Err(GetDeckProfilesError::SessionExpired);
        };
        auth_client.read().get_deck_profiles(&s).await
    });
```

**Why This Broke:**
1. `use_resource` tracks signal reads just like `use_effect`
2. Resource reads `session` → registers as dependency
3. `check_session()` updates `session` → triggers resource refetch
4. Refetch reads `session` again → triggers another refetch
5. Infinite loop: fetch → update → refetch → update → refetch...

**First Attempted Fix (WRONG):**

```rust
// Tried removing check_session() call entirely
let decks: Resource<...> = use_resource(move || async move {
    // No validation before API call - backend will reject expired tokens
    let Some(s) = session.read().clone() else {
        return Err(GetDeckProfilesError::SessionExpired);
    };
    auth_client.read().get_deck_profiles(&s).await
});
```

**Problem with First Fix:**
- Frontend doesn't proactively refresh tokens
- Backend rejects expired tokens with 401
- Need rotating refresh tokens to be updated in Signal for persistence

**Correct Fix:**

```rust
let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
    use_resource(move || async move {
        let Some(sess) = session.read().clone() else {
            return Err(GetDeckProfilesError::SessionExpired);
        };
        
        // Check expiration and refresh if needed
        let Some(active_sess) = auth_client.read().infallible_get_active_session(&sess).await else {
            return Err(GetDeckProfilesError::SessionExpired);
        };
        
        // Make the API call with valid session
        let result = auth_client.read().get_deck_profiles(&active_sess).await;
        
        // Update signal ONLY if session changed (refresh happened)
        // This triggers ONE re-run, then stabilizes
        if active_sess != sess {
            session.set(Some(active_sess));
        }
        
        result
    });
```

**Why This Works:**
1. First run: Reads session, refreshes if needed, makes API call
2. If refresh happened: Updates signal with new tokens
3. Signal update triggers **one** resource re-run
4. Second run: Reads session again, but `active_sess == sess` (no refresh needed)
5. No signal update → no more re-runs → **stabilizes**

The `if active_sess != sess` guard prevents infinite loops by only updating when tokens actually change.

---

## Background: The Session Update Requirement

**Why We Can't Skip Session Updates:**

Your backend uses **rotating refresh tokens** for security:
- Each token refresh generates a **new** refresh token
- Old refresh token is invalidated
- Frontend **must** update stored session with new tokens
- Session persisted to keychain/keystore for cross-session auth

**Without Signal Updates:**
```
1. User logs in → session with refresh_token_A
2. Token expires, refresh happens → new refresh_token_B
3. But frontend still has refresh_token_A in memory
4. Next refresh attempt uses refresh_token_A → 401 Unauthorized
5. User forced to re-login unnecessarily
```

**With Signal Updates:**
```
1. User logs in → session with refresh_token_A stored in Signal
2. Token expires, refresh happens → new refresh_token_B
3. Signal updated with refresh_token_B
4. Signal persisted to keychain for next app launch
5. Seamless experience across sessions
```

---

## Key Dioxus Reactivity Concepts

### 1. `use_effect` Tracks Signal Reads

```rust
use_effect(move || {
    let value = my_signal.read();  // REGISTERS as dependency
    // Effect re-runs whenever my_signal changes
});
```

**When to use:**
- React to signal changes (update UI, trigger side effects)
- Short-lived operations, not infinite loops

**When NOT to use:**
- Long-running background tasks
- Operations that update the signals they read

### 2. `use_future` Runs Once, Ignores Reactivity

```rust
use_future(move || async move {
    loop {
        let value = my_signal.read();  // Does NOT register as dependency
        // Loop runs forever, unaffected by signal changes
    }
});
```

**When to use:**
- Background services (WebSocket polling, periodic timers)
- Infinite loops that should spawn once and run continuously
- Tasks that both read AND write the same signals

### 3. `use_resource` Tracks Dependencies for Data Fetching

```rust
let data: Resource<T> = use_resource(move || async move {
    let param = my_signal.read();  // REGISTERS as dependency
    fetch_data(param).await  // Refetches when my_signal changes
});
```

**When to use:**
- Async data fetching that should re-run on dependency changes
- API calls triggered by user input or state changes

**Gotcha:**
- If you update a signal you read, you create a loop
- Use conditional updates: `if new_value != old_value { signal.set(new_value) }`

### 4. Breaking Reactivity with `spawn()`

```rust
use_effect(move || {
    // Signal reads here ARE tracked
    let trigger = trigger_signal.read();
    
    spawn(async move {
        // Signal reads here are NOT tracked
        let value = data_signal.read();
        // Can safely read/write without affecting effect dependencies
    });
});
```

**Pattern:**
- Read "trigger" signals outside `spawn()` to track them
- Read/write "data" signals inside `spawn()` to avoid tracking

---

## Comparison: Effect vs Future vs Resource

| Hook | Tracks Signals | Re-runs | Use Case |
|------|---------------|---------|----------|
| `use_effect` | ✅ Yes | On dependency change | React to state changes |
| `use_future` | ❌ No | Once on mount | Background services, infinite loops |
| `use_resource` | ✅ Yes | On dependency change | Data fetching, API calls |
| `spawn()` | ❌ No | Manual call | One-shot async tasks |

---

## Learning Areas to Study

### 1. **Dioxus Reactivity System** ⭐⭐⭐ (Critical)

**Current Gap:** Misunderstood when hooks track signal dependencies

**Study:**
- [Dioxus Hooks Documentation](https://dioxuslabs.com/learn/0.6/reference/hooks)
- [Dioxus use_effect Reference](https://dioxuslabs.com/learn/0.6/reference/use_effect)
- [Dioxus use_resource Reference](https://dioxuslabs.com/learn/0.6/reference/resource)
- [Dioxus use_future (Spawn) Reference](https://dioxuslabs.com/learn/0.6/reference/spawn)

**Key Questions to Answer:**
- When does a hook track signal reads as dependencies?
- How does `spawn()` break reactivity tracking?
- When should I use `use_effect` vs `use_future`?

### 2. **Reactive Programming Patterns** ⭐⭐ (Important)

**Current Gap:** Understanding read/write loops in reactive systems

**Study:**
- React useEffect dependency tracking (similar concepts)
- Vue.js computed properties and watchers
- General reactive programming principles

**Key Concepts:**
- Dependency tracking
- Avoiding circular dependencies
- Derived state vs side effects

### 3. **Async Rust in UI Frameworks** ⭐⭐ (Important)

**Current Gap:** When to spawn tasks vs use hooks

**Study:**
- Task spawning patterns in Dioxus
- Differences between `spawn()`, `use_future`, and `use_coroutine`
- Background task lifecycle management

**Key Questions:**
- Do spawned tasks survive component unmounts?
- How do I cancel background tasks?
- When should I use coroutines vs futures?

### 4. **Debugging Reactive Systems** ⭐ (Useful)

**Skills Developed This Session:**
- Adding strategic logging to track effect runs
- Identifying exponential growth patterns
- Isolating which component causes loops

**Further Study:**
- Browser dev tools for performance profiling
- Dioxus-specific debugging techniques
- Memory leak detection in long-running tasks

---

## Practical Rules Going Forward

### ✅ DO:

```rust
// Use use_future for background loops
use_future(move || async move {
    loop {
        // Safe to read/write signals here
    }
});

// Conditionally update signals to prevent loops
if new_value != old_value {
    signal.set(new_value);
}

// Wrap signal reads in spawn() inside effects to avoid tracking
use_effect(move || {
    spawn(async move {
        let value = signal.read();  // Not tracked
    });
});
```

### ❌ DON'T:

```rust
// DON'T: Read and write same signal in use_effect
use_effect(move || {
    let value = signal.read();  // Reads
    signal.set(value + 1);      // Writes → triggers effect again
});

// DON'T: Spawn infinite loops in use_effect
use_effect(move || {
    spawn(async move {
        loop {
            // This spawns a NEW loop every time effect runs
        }
    });
});

// DON'T: Update signals unconditionally in use_resource
use_resource(move || async move {
    let value = signal.read();
    signal.set(new_value);  // Always triggers refetch
});
```

---

## Code Examples from This Project

### ✅ Correct: Background Session Refresh

```rust
// home.rs, profile.rs - Background validation loop
use_future(move || async move {
    let mut interval = interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let Some(sess) = session.read().clone() else {
            continue;
        };
        // Check and only update if changed
        if let Some(active_sess) = auth_client.read().infallible_get_active_session(&sess).await {
            if active_sess != sess {
                session.set(Some(active_sess));
            }
        }
    }
});
```

### ✅ Correct: Resource with Session Refresh

```rust
// decks.rs - Fetch decks with token validation
let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
    use_resource(move || async move {
        let Some(sess) = session.read().clone() else {
            return Err(GetDeckProfilesError::SessionExpired);
        };
        
        // Validate and refresh if needed
        let Some(active_sess) = auth_client.read().infallible_get_active_session(&sess).await else {
            return Err(GetDeckProfilesError::SessionExpired);
        };
        
        let result = auth_client.read().get_deck_profiles(&active_sess).await;
        
        // Conditional update prevents loop
        if active_sess != sess {
            session.set(Some(active_sess));
        }
        
        result
    });
```

### ✅ Correct: One-shot Async in Effect

```rust
// login.rs - Submit on swipe detection
use_effect({
    let mut s = swipe_state.clone();
    let c = swipe_config.clone();
    move || {
        // Read outside spawn to track dependency
        if s.read().latest_swipe == c.submission_swipe && c.submission_swipe.is_some() {
            s.write().latest_swipe = None;
            submit_attempted.set(true);
            is_loading.set(true);

            // Spawn one-shot async task
            spawn(async move {
                // Validation and HTTP request
                // ...
                session.set(Some(new_session));
                is_loading.set(false);
            });
        }
    }
});
```

---

## Testing Your Understanding

### Question 1:
What's wrong with this code?

```rust
use_effect(move || {
    let count = counter.read();
    counter.set(count + 1);
});
```

<details>
<summary>Answer</summary>

Infinite loop. Effect reads `counter`, then updates it, triggering the effect to run again. This increments counter infinitely until overflow.

**Fix:** Use `use_future` or add a condition to stop updates.
</details>

### Question 2:
Why does this work without infinite loops?

```rust
use_resource(move || async move {
    let value = signal.read();
    if value.needs_refresh() {
        let new_value = refresh().await;
        if new_value != value {
            signal.set(new_value);
        }
    }
    value
});
```

<details>
<summary>Answer</summary>

The `if new_value != value` check prevents updates when value hasn't changed. After refresh, second run sees matching values and doesn't update signal, breaking the loop.
</details>

### Question 3:
When should you use `use_future` instead of `use_effect`?

<details>
<summary>Answer</summary>

Use `use_future` for:
- Infinite background loops (timers, polling)
- Tasks that should run once and continue forever
- Operations that both read AND write the same signals

Use `use_effect` for:
- Reacting to specific signal changes
- Short-lived side effects
- Syncing derived state
</details>

---

## Summary

**The Bug:** `use_effect` spawning infinite loops + `use_resource` updating signals it reads = exponential task explosion

**The Fix:** 
1. Replace `use_effect` → `use_future` for background loops
2. Add conditional signal updates: `if new != old { signal.set(new) }`

**The Lesson:** Understand which hooks track reactivity and avoid read/write loops in reactive contexts.

**Next Steps:**
1. Study Dioxus hooks documentation thoroughly
2. Practice identifying reactivity loops in code reviews
3. Apply conditional update pattern everywhere signals are updated

---

**Related Files:**
- `zwiper/src/lib/inbound/ui/components/screens/app/home.rs`
- `zwiper/src/lib/inbound/ui/components/screens/app/profile.rs`
- `zwiper/src/lib/inbound/ui/components/screens/app/decks.rs`
- `zwiper/src/lib/inbound/ui/components/screens/auth/login.rs`
- `zwiper/src/lib/outbound/client/auth/session.rs`

