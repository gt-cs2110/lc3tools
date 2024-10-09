# LC3Tools Infrastructure & Developing Notes

This document discusses the build infrastructure of this repository and other various notes about development.

## Project Structure

This repository is broken up into `src/gui` (the frontend) and `src/backend` (take a guess what that is).

### `src/backend`

This project is a [Neon project](https://neon-rs.dev) which acts as glue around the `lc3-ensemble` engine. Ensemble's source can be found [here](https://github.com/endorpersand/lc3-ensemble).

### `src/gui`

This is where the Electron handling and rendering occurs. This project uses [Electron Forge](https://www.electronforge.io), Vite, and Vue 3 and is subject to all of the quirks that come with that.

`src/gui` can be seen as the main folder for the project, as all of the building (including the building of the backend) can be done here.

## Build Scripts

In `src/gui`, there are a few notable build scripts.

- `npm run build-backend`: This builds the backend. This **must** be run before running any of the other Electron-loading scripts, otherwise the backend will not be present.

For development,

- `npm run start`: Opens a development build.
- `npm run lint`: Calls `eslint`.

For building and deployment,

- `npm run package`: Creates an executable, which can be ran locally.
- `npm run make`: Creates a distributable.
- `npm run publish`: Creates a distributable and publishes it to GitHub. This never needs to be called as it is handled by the [Github Workflow](#github-ci).

### Build Script Notes

`npm run build-backend`: This script is defined as...

```sh
npm run --prefix ../backend build && npm install --install-links lc3-backend@../backend
```

This runs the `build` script on the backend (without having to `cd` to it) and then reinstalls the dependency (which adds the dylib in `node_modules`).

[Due to a quirk of Electron Forge and Vite](https://github.com/electron/forge/issues/3624), we use the `--install-links` flag so that a copy of the data is bundled into the application (rather than a symlink). If this ever gets fixed, then the script should be reduced to simply `npm run --prefix ../backend build`.

## Distributables

This project releases several different types of distributables for different platforms (which can be configured in `forge.config.ts`). The ones *intended* for distribution are:

- [Squirrel.Windows](https://www.electronforge.io/config/makers/squirrel.windows) (Windows)
- [DMG](https://www.electronforge.io/config/makers/dmg) (macOS)
- [Flatpak](https://www.electronforge.io/config/makers/flatpak) (Linux)
- [ZIP](https://www.electronforge.io/config/makers/zip) (Linux and macOS*)

\* *`.zip`s are also provided for macOS, but the `.dmg`s are heavily preferred. These `.zip`s are moreso for facilitating auto-updates rather than distribution.*

### On Linux

The preferred Linux distributable is `.flatpak`.

AppImages are not supported out-of-the-box on Electron Forge, and we do not wish to maintain additional `.deb` and `.rpm` executables.

*That being said*, the `.flatpak` maker provided by Electron Forge has a lot of quirks, so here's some notes.

Here is every dependency required by Electron Forge's `.flatpak` maker:

- Installed with the system's package manager
  - `flatpak`
  - `flatpak-builder`
  - `eu-strip` (from `elfutils`)
- Flatpak dependencies
  - `org.freedesktop.Platform`
  - `org.freedesktop.SDK`
  - `org.electronjs.Electron2.BaseApp`
- Other dependencies
  - [zypak](https://github.com/refi64/zypak)

Technically only the system dependencies are truly required as it will try to install the other dependencies if not present on the system; however,

- the maker only recognizes specific versions of `org.freedesktop.Platform` and `org.freedesktop.SDK` (defined in `forge.config.ts`)
- the maker will only succeed in installing the dependencies if `flathub` is present as a user remote
- the maker also does `git` schenanigans to properly install `zypak`

Refer to `forge.config.ts` for the configured versions of `org.freedesktop.Platform`, `org.freedesktop.SDK`, and `zypak`. These should be updated when the current configured version goes to EOL.

Finally, if you wish to debug what is going on with the Electron Forge maker, set the `DEBUG` environment variable:

```sh
DEBUG=* npm run make
```

## Github CI

This repository uses a Github workflow to perform cross-compilation and macOS code-signing for releases.

It creates a new build of Windows, Linux, and macOS distributables and publishes it to Github Releases when a tag of the form `vX.X.X` is created.

Note that the workflow will publish to the release which matches the `package.json` version of `LC3Tools`. If you wish to create new builds, you should:

1. Create a commit bumping the `package.json` version.
2. Trigger the workflow by:
   - creating a new tag of the form `vX.X.X`,
   - creating a new release with a tag on that commit,
   - or by triggering `workflow_dispatch`
3. If the release is in draft/prerelease mode, then fix the settings! (Also if a new tag is created, be sure to cancel the workflow).

Once builds have been published to the Releases page, [`update-electron-app`](https://github.com/electron/update-electron-app) informs all users to update with the new build!