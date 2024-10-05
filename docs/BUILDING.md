# Building

This document describes instructions to manually build executables for use on a single computer.

Before resorting to this, please refer to [the installation guide](./INSTALL.md) to see the available prebuilt binaries and troubleshooting steps.

This document is a brief discussion of how to build an executable and does not go into significant detail about the build system or general infrastructure of this repository. To learn more about that, refer to [the developing guide](./DEVELOPING.md).

## Requirements

You will need:

- Cargo
- Node and `npm`

No specific version of the above dependencies are required, but as of writing, builds have been made with: Cargo 1.81.0, Node v20.17.0, npm 10.8.2.

### Steps

To create an executable, you must be in the `src/gui` directory. You can then build with:

```sh
npm install
npm run build-backend
npm run package
```

The packaged executable can be found in `src/gui/out`.

## Troubleshooting Manual Builds

GitHub Actions handles most of these issues for the released builds, but you may run across these issues when trying to build manually.

### macOS: `LC3Tools.app: code has no resources but signature indicates they must be present` or related code-signing/notarization error on packaging

This occurs because the packager will attempt to sign and notarize on macOS even if the necessary certificates are not present.

To fix this, remove the `osxSign` and `osNotarize` config entries in `forge.config.ts`.

This will introduce secondary errors (particularly the next error below), but it will at least build.

### macOS: `Operation not permitted (os error 1)` on loading a file

For unsigned macOS builds, the Rust library will not be able to access the filesystem (preventing it from doing anything). This can be fixed by adding an ad-hoc code signature:

```sh
codesign --force --deep -s - ./out/*/LC3Tools.app
```

This adds a placeholder signature internally to all files in the executable, which allows the Rust library to access the filesystem.

### macOS: `"LC3Tools.app" is damaged and can't be opened. You should move it to the Trash.` on opening LC3Tools

This will occur if an unsigned version of the executable was installed from the internet or from a ZIP file.

It is not damaged. It is missing a signature, which ARM macOS claims is file damage. Same fix above can be applied, or this fix can be done to wipe the quarantine:

```sh
xattr -dr com.apple.quarantine /Applications/LC3Tools.app
```

Note that wiping the quaratine like this may still result in the `Operation not permitted (os error 1)` error.

## Creating a Distributable

This section is for users who would like to build a distributable which can be shared to other computers. Alongside all of the [requirements for executables](#requirements), you may need additional dependencies depending on the type of distributable:

| Distributable    | Dependencies                             |
|------------------|------------------------------------------|
| macOS `.dmg`     | Python and `python-setuptools`           |
| Linux `.flatpak` | `flatpak`, `flatpak-builder`, `elfutils` |

Again, no specific version of the above dependencies are required.

For Flatpak builds, ensure you also have a user remote to `flathub`:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

## Distributable Steps

To create an executable, you must be in the `src/gui` directory. Then, you can build with:

```sh
npm install
npm run build-backend
npm run make # <-- the only difference
```

Once those three scripts are run, you're done! The can be found in `src/gui/out/make`.
