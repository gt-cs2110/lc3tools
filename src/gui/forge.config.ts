import type { ForgeConfig } from '@electron-forge/shared-types';
import { MakerDeb } from "@electron-forge/maker-deb";
import { MakerDMG } from "@electron-forge/maker-dmg";
import { MakerFlatpak } from "@electron-forge/maker-flatpak";
import { MakerRpm } from "@electron-forge/maker-rpm";
import { MakerSquirrel } from '@electron-forge/maker-squirrel';
import { MakerZIP } from "@electron-forge/maker-zip";
import { PublisherGithub } from '@electron-forge/publisher-github';
import { AutoUnpackNativesPlugin } from '@electron-forge/plugin-auto-unpack-natives';
import { VitePlugin } from '@electron-forge/plugin-vite';
import { FusesPlugin } from '@electron-forge/plugin-fuses';
import { FuseV1Options, FuseVersion } from '@electron/fuses';
import fs from "node:fs/promises";
import path from "node:path";

function osxSignNotarize() {
  const options: {
    osxSign?: ForgeConfig["packagerConfig"]["osxSign"],
    osxNotarize?: ForgeConfig["packagerConfig"]["osxNotarize"]
  } = {};

  if (process.env.SIGNING_IDENTITY) {
      options.osxSign = {
      identity: process.env.SIGNING_IDENTITY,
      preAutoEntitlements: false,
      optionsForFile: () => ({
          entitlements: "entitlements.plist",
      }),
    };
  } else {
    console.warn("Missing environment variables -- skipping macOS signing...");
  }
  if (process.env.NOTARIZE_EMAIL && process.env.NOTARIZE_PASSWORD && process.env.TEAM_ID) {
    options.osxNotarize = {
      appleId: process.env.NOTARIZE_EMAIL,
      appleIdPassword: process.env.NOTARIZE_PASSWORD,
      teamId: process.env.TEAM_ID,
    };
  } else {
    console.warn("Missing environment variables -- skipping macOS notarizing...");
  }

  return options;
}

const config: ForgeConfig = {
  packagerConfig: {
    asar: true,
    icon: "static/icons/icon",
    ...osxSignNotarize(),
  },
  hooks: {
    packageAfterCopy: async (_cfg, buildPath) => {
      const nativeDeps = {
        "lc3-backend": ["index.node", "package.json"]
      };

      // Copy all native modules to ASAR, so they can be accessed in build
      for (const [dep, filenames] of Object.entries(nativeDeps)) {
        const src = path.join("node_modules", dep);
        const dst = path.join(buildPath, "node_modules", dep);
        for (const f of filenames) {
          await fs.cp(path.join(src, f), path.join(dst, f), {
            recursive: true,
            dereference: true
          });
        }
      }
    }
  },
  rebuildConfig: {},
  makers: [
    new MakerSquirrel({}), 
    new MakerDMG(), 
    new MakerZIP({}, ['darwin', 'linux']), 
    new MakerDeb({}),
    new MakerRpm({}),
    new MakerFlatpak({
      // Override the default settings:
      // Uses `org.freedesktop.Platform//24.08` and `org.freedesktop.SDK//24.08` instead of `19.08`
      // Uses zypak v2024.01.17 instead of the default (v2021).
      options: {
        runtimeVersion: "24.08",
        files: [],
        modules: [
          {
            name: "zypak",
            sources: [
              {
                type: "git",
                url: "https://github.com/refi64/zypak",
                tag: "v2024.01.17"
              }
            ]
          }
        ],
        id: "io.github.gt_cs2110.lc3tools"
      }
    })
  ],
  publishers: [new PublisherGithub({
    repository: {
      owner: "gt-cs2110",
      name: "lc3tools"
    },
    prerelease: true,
    draft: true
  })],
  plugins: [
    new AutoUnpackNativesPlugin({}),
    new VitePlugin({
      // `build` can specify multiple entry builds, which can be Main process, Preload scripts, Worker process, etc.
      // If you are familiar with Vite configuration, it will look really familiar.
      build: [
        {
          // `entry` is just an alias for `build.lib.entry` in the corresponding file of `config`.
          entry: 'src/main.ts',
          config: 'vite.main.config.mts',
          target: 'main',
        },
        {
          entry: 'src/preload.ts',
          config: 'vite.preload.config.mts',
          target: 'preload',
        },
      ],
      renderer: [
        {
          name: 'main_window',
          config: 'vite.renderer.config.mts',
        },
      ],
    }),
    // Fuses are used to enable/disable various Electron functionality
    // at package time, before code signing the application
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};

export default config;
