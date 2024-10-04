# Building

LC3Tools can be manually built (if the prebuilt binaries are not sufficient).

Refer to [INSTALL.md](./INSTALL.md) to see the available prebuilt binaries and troubleshooting steps.

## Requirements

- Cargo
- Node
- Python and `python-setuptools` (macOS)
- `flatpak`, `flatpak-builder`, `elfutils` (Linux)

Most modern versions of the above should work here, but as of writing, it is known to work on: Cargo 1.80.0, Node 22.8.0, and Python 3.12.

(For Linux, make sure to setup the `flathub` repo: `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`).

## Steps

To build, you must be in the `src/gui` directory. Then, building is three steps:

```sh
npm install
npm run build-backend
npm run package
```

(Note that the `build-backend` script requires Cargo and `package` requires Python on macOS).

Once those three scripts are run, you're done! The packaged build should be found in `src/gui/out`.

### Notes

These are some notes about what the scripts above do. This is not at all necessary to read if you're simply trying to build an executable.

`npm install`: This command installs all the required dependencies for LC3Tools, which is pretty straightforward.

`npm run build-backend`: This script is equivalent to

```sh
npm run --prefix ../backend build && npm install --install-links lc3-backend@../backend
```

This runs the build script on the engine package (creating a dylib) and then reinstalls the dependency (in order to allow LC3Tools to use that dylib). Note that [due to an Electron Forge bug](https://github.com/electron/forge/issues/3624), we have to use the `--install-links` flag to make a direct copy of the data. If this bug ever gets fixed, this script should be able to be reduced to simply `npm run --prefix ../backend build`.

`npm run package`: This command relies on Electron Forge's package utilities. It will make an executable and then package it into a distributable format. If you don't need to package it and would rather just have the executable directly, you can instead do `npm run make`.

## Troubleshooting Builds

These fixes are handled by GitHub Actions, but you may run across them trying to build the files manually.

### macOS: `Operation not permitted (os error 1)` on loading a file

If there's no code signature, the Rust library is unable to access the filesystem. This can be fixed by adding an ad-hoc code signature:

```sh
codesign --force --deep -s - ./out/*/LC3Tools.app
```

This adds a placeholder signature internally to all files in the executable, which allows the Rust library to access the filesystem.

### macOS: `"LC3Tools.app" is damaged and can't be opened. You should move it to the Trash.` on opening LC3Tools

It is not damaged. It is missing a signature, which ARM macOS interprets as file damage. This occurs specifically when there is a missing signature and the app was installed from the internet.

Same fix above can be applied, or this fix can be done to wipe the quarantine:

```sh
xattr -dr com.apple.quarantine /Applications/LC3Tools.app
```

Note that the `Operation not permitted (os error 1)` error can still occur.
