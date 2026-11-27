# Web

`gate.sh` is zwiper-web's check before a commit: nightly fmt, clippy for the workspace and for wasm, and the shared-sheet copies, which it rewrites and then stops so you commit them. Run it from anywhere in the checkout; `gate.sh dx` adds a `dx build`.
