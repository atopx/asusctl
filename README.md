# `asusctl` for ASUS ROG

[Become a Patron!](https://www.patreon.com/bePatron?u=7602281) - [Asus Linux Website](https://asus-linux.org/)

**WARNING:** Many features are developed in tandem with kernel patches. If you see a feature is missing you either need a patched kernel or latest release.

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

Now includes a GUI, `rog-control-center`.

## Kernel support

**The minimum supported kernel version is 6.10**, which will contain the patches from [here](https://lore.kernel.org/platform-driver-x86/20240404001652.86207-1-luke@ljones.dev/). This is especially required for 2023+ devices and possibly some lat 2022 devices.

Z13 devices will need [these](https://lore.kernel.org/linux-input/20240416090402.31057-1-luke@ljones.dev/T/#t)

## X11 support

X11 is not supported at all, as in I will not help you with X11 issues if there are any due to limited time and it being unmaintained itself. You can however build `rog-control-center` with it enabled `cargo build --features "rog-control-center/x11"`.

## Goals

The main goal of this work is to provide a safe and easy to use abstraction over various laptop features via DBUS, and to provide some helpful defaults and other behaviour such as toggling throttle/profile on AC/battery change.

1. Provide safe dbus interface
2. Respect the users resources: be small, light, and fast

Point 4? asusd currently uses a tiny fraction of cpu time, and less than 1Mb of ram, the way
a system-level daemon should. Languages such as JS and python should never be used for system level daemons (please stop).

## Keyboard LEDs

The level of support for laptops is dependent on folks submitting data to include in [`./rog-aura/data/layouts/aura_support.ron`](./rog-aura/data/layouts/aura_support.ron), typically installed in `/usr/share/asusd/aura_support.ron`. This is because the controller used for keyboards and LEDs is used across many years and many laptop models, all with different firmware configurations - the only way to track this is with the file mentioned above. Why not just enable all by default? Because it confuses people.

See the [rog-aura readme](./rog-aura/README.md) for more details.

## Discord

[![Discord](https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/z8y99XqPb7)

## SUPPORTED LAPTOPS

Most ASUS gaming laptops that have a USB keyboard. If `lsusb` shows something similar
to this:

```
Bus 001 Device 002: ID 0b05:1866 ASUSTek Computer, Inc. N-KEY Device
```

or

```
Bus 003 Device 002: ID 0b05:19b6 ASUSTek Computer, Inc. [unknown]
```

then it may work without tweaks. Technically all other functions except the LED
and AniMe parts should work regardless of your latop make.

## Implemented

The list is a bit outdated as many features have been enabled in the Linux kernel with upstream patches and then supported in asusctl suite.

- [x] System daemon
- [x] GUI app (includes tray and notifications)
- [x] Setting/modifying built-in LED modes
- [x] Per-key LED setting
- [x] Fancy LED modes (See examples) (currently being reworked)
- [x] AniMatrix display on G14 and M16 models that include it
- [x] Set battery charge limit (with kernel supporting this)
- [x] Fan curve control on supported laptops (G14/G15, some TUF like FA507)
- [x] Toggle bios setting for boot/POST sound
- [x] Toggle GPU MUX (g-sync, or called MUX on 2022+ laptops)

# GUI

A gui is now in the repo - ROG Control Center. At this time it is still a WIP, but it has almost all features in place already.

**NOTE**: Xorg is not supported.

# BUILDING

Rust and cargo are required, they can be installed from [rustup.rs](https://rustup.rs/) or from the distro repos if newer than 1.75.

**fedora:**

    dnf install cmake clang-devel libinput-devel libseat-devel libgbm-devel libxkbcommon-devel systemd-devel libdrm-devel expat-devel pcre2-devel libzstd-devel gtk3-devel
    make
    sudo make install

**openSUSE:**

Works with KDE Plasma (without GTK packages)

    zypper in -t pattern devel_basis
    zypper in rustup make cmake clang-devel libinput-devel libseat-devel libgbm-devel libxkbcommon-devel systemd-devel libdrm-devel expat-devel pcre2-devel libzstd-devel gtk3-devel
    make
    sudo make install

**Debian(unsuported):**

officially unsuported,but you can still try and test it by yourself(some features may not be available).

    sudo apt install libclang-dev libudev-dev libfontconfig-dev build-essential cmake libseat-dev libxkbcommon-dev libinput-dev libgbm-dev
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    make
    sudo make install

**Ubuntu, Popos (unsuported):**

instructions removed as outdated

## Installing

- Fedora copr = https://copr.fedorainfracloud.org/coprs/lukenukem/asus-linux/
- openSUSE = https://download.opensuse.org/repositories/home:/luke_nukem:/asus/

=======

The default init method is to use the udev rule, this ensures that the service is
started when the device is initialised and ready.

You may also need to activate the service for debian install. If running Pop!\_OS, I suggest disabling `system76-power` gnome-shell extension and systemd service.

## Upgrading

If you are upgrading from a previous installed version, you will need to restart the service or reboot.

```
$ systemctl daemon-reload && systemctl restart asusd
```

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd/`.

# Contributing

See `CONTRIBUTING.md`. Additionally, also do `cargo clean` and `cargo test` on first checkout to ensure the commit hooks are used (via `cargo-husky`).

Generation of the bindings with `make bindings` requires `typeshare` to be installed.

Dbus introsepction XML requires with `make introspection` requires `anime_sim` to be running before starting `asusd`.

# OTHER

## AniMe Matrix simulator

A simulator using SDL2 can be built using `cargo build --package rog_simulators` and run with `./target/debug/anime_sim`. Once started `asusd` will need restarting to pick it up. If running this sim on a laptop _with_ the display, the simulated display will be used instead of the physical display.

## Supporting more laptops

Please file a support request.

# License & Trademarks

Mozilla Public License 2 (MPL-2.0)

---

ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.

Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.

The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.

---
