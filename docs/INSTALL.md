# Installation

This document describes steps to install LC3Tools.

Check out the [releases page](https://github.com/gt-cs2110/lc3tools/releases) to see recent prebuilt binaries for various systems. The following are provided:

| Platform      | File                                                 |
|---------------|------------------------------------------------------|
| Windows       | `LC3Tools-<version>.Setup.exe`                       |
| macOS (Intel) | `LC3Tools-<version>-x64.dmg`                         |
| macOS (ARM)   | `LC3Tools-<version>-arm64.dmg`                       |
| Linux         | `io.github.gt_cs2110.lc3tools_stable_x86_64.flatpak` |

If a version for your operating system is not available (or does not work), you can manually build LC3Tools. Instructions to do so can be found [here](./BUILDING.md).

Alternatively, if you believe it is extremely important to have support for your specific platform, feel free to submit an issue or a pull request!

## Troubleshooting Installs

You *unfortunately* may get some errors on installation. Here's some tips:

### Windows: Windows Defender Activation

The Windows build of LC3Tools is *not* code-signed. This means it may trigger Windows Defender or a third-party anti-virus.

LC3Tools is not a virus. You should be able to bypass this message by going into `More Details` > `Run anyway`.

### Linux: Something doesn't work

Oops.

## Uninstalling

If you wish to *fully* uninstall LC3Tools, removing any reference of it from your computer:

### Uninstalling on Windows

The installation process is described on [the Squirrel.Windows framework docs](https://github.com/Squirrel/Squirrel.Windows/blob/develop/docs/using/install-process.md). You can follow that to understand what the `Setup.exe` installer does. However, briefly,

1. Delete `LC3Tools-<version>.Setup.exe` (this is the installer).
2. Delete the `%LocalAppData%\LC3Tools\` folder (this holds all installs of LC3Tools).
3. Delete Desktop and Windows Start Menu scripts.
4. Delete the `%APPDATA%\LC3Tools\` folder (this holds config files for LC3Tools).

### Uninstalling on macOS

1. Delete `LC3Tools` from your `Applications` folder.
2. Delete the `~/Library/Application Support/LC3Tools/` folder (this holds config files for LC3Tools).

### Uninstalling on Linux

1. Delete the application itself.
2. Delete `$XDG_CONFIG_HOME/LC3Tools/` or `~/.config/LC3Tools` (this holds config files for LC3Tools).
