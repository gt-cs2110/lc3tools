# Building

LC3Tools can be manually built (if the prebuilt binaries are not sufficient).

Refer to [INSTALL.md](./INSTALL.md) to see the available prebuilt binaries and troubleshooting steps.

## Requirements

- Cargo
- Node
- Python and `python-setuptools` (macOS)

Most modern versions of the above should work here, but as of writing, it is known to work on Cargo 1.80.0, Node 22.8.0, and Python 3.12.

## Steps

1. First, you need to build the backend LC3 engine. You can do so with:
   - `cd src/backend`
   - `npm install`
   - `npm run build`

    (This step requires Node and Cargo.)
2. Once the backend is built, you can build the frontend and package it.
   - `cd src/gui`
   - `npm install --install-links` (flag that lets Vite to behave with `lc3-backend`)
   - `npm run package`

   (This step requires Python on macOS.)
3. And, you're done! The packaged build should be found in `src/gui/out/make`.
