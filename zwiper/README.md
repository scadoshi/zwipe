# zwiper

dioxus frontend for zwipe - mobile-first mtg deck builder with swipe navigation.

## structure

```
zwiper/
├── assets/          # css, favicon, static files
├── src/
│   ├── bin/
│   │   └── main.rs  # app entry point
│   └── lib/
│       ├── domain/       # errors, helpers, types
│       ├── inbound/ui/   # dioxus components (auth, app, swipe)
│       └── outbound/     # http client, session storage
├── build.rs         # compile-time config (backend url)
└── Dioxus.toml      # platform config, bundle settings
```

## development

```bash
# serve with hot reload (default: web)
dx serve

# specific platform
dx serve --platform desktop
dx serve --platform ios     # requires xcode
dx serve --platform android # requires android sdk

# build release
dx build --release --platform desktop
```

## current status

- swipe gesture system complete (touch + mouse)
- auth screens with validation
- http client integration
- session management via context
- debugging: screen freeze after login (session persistence issue)

## architecture

hexagonal pattern matching backend:
- **inbound/ui/** - dioxus components and screens
- **outbound/** - http client, session storage (keychain/keystore)
- **domain/** - shared types via feature flags from zerver

session storage uses keyring crate for ios keychain/android keystore (requires entitlements for production).

## environment

requires `.env` file with:
```
BACKEND_URL=http://localhost:3000
RUST_LOG=info
```

backend url is baked into binary at compile-time via build.rs.
