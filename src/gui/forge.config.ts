import type { ForgeConfig } from '@electron-forge/shared-types';
import { MakerSquirrel } from '@electron-forge/maker-squirrel';
import { MakerDMG } from "@electron-forge/maker-dmg";
import { MakerFlatpak } from "@electron-forge/maker-flatpak";
import { PublisherGithub } from '@electron-forge/publisher-github';
import { VitePlugin } from '@electron-forge/plugin-vite';
import { FusesPlugin } from '@electron-forge/plugin-fuses';
import { FuseV1Options, FuseVersion } from '@electron/fuses';

const config: ForgeConfig = {
  packagerConfig: {
    asar: true,
    icon: "static/icons/icon"
  },
  rebuildConfig: {},
  makers: [new MakerSquirrel({}), new MakerDMG(), new MakerFlatpak({
    // Override the default settings:
    // Uses `org.freedesktop.Platform//24.08` and `org.freedesktop.SDK//24.08` instead of `19.08`
    // Removes external dependency zypak, since it's present on `org.electronjs.Electron2.BaseApp`
    options: {
      runtimeVersion: "24.08",
      files: [],
      modules: [
        {
          name: "zypak",
          "sources": [
            {
              type: "git",
              url: "https://github.com/refi64/zypak",
              tag: "v2024.01.17"
            }
          ]
        }
      ]
    }
  })],
  publishers: [new PublisherGithub({
    repository: {
      owner: "endorpersand",
      name: "lc3tools"
    },
    prerelease: true,
    draft: true
  })],
  plugins: [
    new VitePlugin({
      // `build` can specify multiple entry builds, which can be Main process, Preload scripts, Worker process, etc.
      // If you are familiar with Vite configuration, it will look really familiar.
      build: [
        {
          // `entry` is just an alias for `build.lib.entry` in the corresponding file of `config`.
          entry: 'src/main.ts',
          config: 'vite.main.config.ts',
        },
        {
          entry: 'src/preload.ts',
          config: 'vite.preload.config.ts',
        },
      ],
      renderer: [
        {
          name: 'main_window',
          config: 'vite.renderer.config.ts',
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
