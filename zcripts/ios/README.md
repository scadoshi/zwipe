# iOS

Simulator and device helpers for zwiper. Each script says its usage at the top.

- `sim.sh` boots the default simulator (iPhone 17 Pro Max) and makes it the only one running.
- `sim_icons.sh` puts the app icons into a simulator build, which `dx serve` leaves out.
- `deploy_debug.sh` installs a debug build on the iPhone on the cable. It talks to a local server, so it's for development only.
- `deploy.sh` builds and installs a release build on the iPhone.
- `icons.sh` compiles the app icons into an asset catalog; `icon.py` renders the per-theme alternates it uses.
- `device.sh` is a shared helper the others source to pick the iPhone. Don't run it on its own.

Store builds and submission steps are in `zwipe-context operations/ios/`.
