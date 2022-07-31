# HyFetch

neofetch with pride flags <3

<img src="https://user-images.githubusercontent.com/22280294/162614541-af2b4660-f1f7-4287-b978-1aa2266ac70f.png" width="60%">

### Running Updated Original Neofetch

This repo also serves as an updated version of the original `neofetch` since the upstream [dylanaraps/neofetch](https://github.com/dylanaraps/neofetch) doesn't seem to be maintained anymore (as of Jul 30, 2022, the original repo hasn't merged a pull request for 6 months). If you only want to use the updated neofetch without pride flags, you can use the `neofetch` script from this repo. To prevent command name conflict, I call it `neowofetch` :)

* Method 1: `pip install hyfetch` then run `neowofetch`
* Method 2: `npx neowofetch`
* Method 3: `bash <(curl -sL neowofetch.hydev.org)`

## Installation

### Method 1: Install using Python pip (Recommended)

Install Python >= 3.7 first. Then, just do:

```sh
pip install hyfetch
```

### Method 2: Install using system package manager

Currently, these distributions have existing packages for HyFetch:

* ArchLinux: `yay -S hyfetch` (Thanks to @ Aleksana)
* Nix: `nix-env -i hyfetch` (Thanks to @ YisuiDenghua)
* Guix: `guix install hyfetch` (Thanks to @ WammKD)

## Usage

When you run `hyfetch` for the first time, it will prompt you to choose a color system and a preset. Just follow the prompt, and everything should work (hopefully). If something doesn't work, feel free to submit an issue!

If you want to use the updated `neofetch` without LGBTQ flags, check out [this section](https://github.com/hykilpikonna/hyfetch#running-updated-original-neofetch)

#### Q: How do I change my config?

A: Use `hyfetch -c`

#### Q: What do I do if the color is too dark/light for my theme?

A: You can try setting the colors' "lightness" in the configuration menu. The value should be between 0 and 1. For example, if you are using dark theme and the rainbow flag is too dark to display, you can set lightness to 0.7.

Feel free to experiment with it!

![image](https://user-images.githubusercontent.com/22280294/162614553-eb758e4e-1936-472c-8ca7-b601c696c6eb.png)

## Change Log

### About Notation

Updates to HyFetch begins with the emoji 🌈  
Updates to `neowofetch` begins with the emoji 🖼️

### TODO

* [ ] Paginate flags
* [ ] Implement light/dark background detection based on https://github.com/muesli/termenv

### Unreleased 1.2.1

<details>
  <summary>🖼️ Ascii Art Changes</summary>  

* Ascii - Improve Trisquel ([dylanaraps#1946](https://github.com/dylanaraps/neofetch/pull/1946))
* Ascii - Improve LangitKetujuh (dylanaraps/neofetch#1948)
* Ascii - Improve Artix small (dylanaraps/neofetch#1872)
* Ascii - Update Archcraft (dylanaraps/neofetch#1919)

</details>

<details>
  <summary>🖼️ Distro/OS Support Changes</summary>  

* OS - Support Old macOS 10.4 and 10.5 ([dylanaraps#2151](https://github.com/dylanaraps/neofetch/pull/2151))
* OS - Identify Hackintosh VM (dylanaraps/neofetch#2005)
* Distro - Fix model detection for Ubuntu Touch ([dylanaraps#2167](https://github.com/dylanaraps/neofetch/pull/2167))
* Distro - Add EncryptOS ([dylanaraps#2158](https://github.com/dylanaraps/neofetch/pull/2158))
* Distro - Add BigLinux ([dylanaraps#2061](https://github.com/dylanaraps/neofetch/pull/2061))
* Distro - Add AmogOS ([dylanaraps#1904](https://github.com/dylanaraps/neofetch/pull/1904))
* Distro - Add CutefishOS ([dylanaraps#2054](https://github.com/dylanaraps/neofetch/pull/2054))
* Distro - Add PearOS ([dylanaraps#2049](https://github.com/dylanaraps/neofetch/pull/2049))
* Distro - Add FusionX ([dylanaraps#2011](https://github.com/dylanaraps/neofetch/pull/2011))
* Distro - Add Q4OS ([dylanaraps#1973](https://github.com/dylanaraps/neofetch/pull/1973))
* Distro - Add CachyOS ([dylanaraps#2026](https://github.com/dylanaraps/neofetch/pull/2026))
* Distro - Add Soda Linux ([dylanaraps#2023](https://github.com/dylanaraps/neofetch/pull/2023))
* Distro - Add Elive Linux ([dylanaraps#1957](https://github.com/dylanaraps/neofetch/pull/1957))
* Distro - Add Uos (dylanaraps/neofetch#1991)
* Distro - Add MassOS (dylanaraps/neofetch#1947)
* Distro - Add CalinixOS (dylanaraps/neofetch#1988)
* Distro - Add Kaisen Linux (dylanaraps/neofetch#1958)
* Distro - Add yiffOS (dylanaraps/neofetch#1920)
* Distro - Add Sulin (dylanaraps/neofetch#1896)
* Distro - Add Wii Linux (dylanaraps/neofetch#1929)
* Distro - Add Linspire (dylanaraps/neofetch#1905)
* Distro - Add Ubuntu Kylin (dylanaraps/neofetch#1974)
* Distro - Add OPNsense (dylanaraps/neofetch#1055)
* Distro - Improve BSD machine arch detection (dylanaraps/neofetch#2015)
* Distro - Improve Manjaro version detection (dylanaraps/neofetch#1879)

</details>

<details>
  <summary>🖼️ Device Support Changes</summary>  

* Terminal - Add Fig ([dylanaraps#2077](https://github.com/dylanaraps/neofetch/pull/2077))
* Terminal - Identify font for Apple Terminal (dylanaraps/neofetch#2017)
* CPU - Identify core count for Apple M1 ([dylanaraps#2038](https://github.com/dylanaraps/neofetch/pull/2038))
* GPU - Identify OpenCL GPU without PCIe (dylanaraps/neofetch#1928)
* Host - Identify MacBook & Update iDevice models (dylanaraps/neofetch#1944)
* Battery - Identify power adapter for MacBooks (dylanaraps/neofetch#1945)
* DE - Identify KF5 and Qt versions for Plasma (dylanaraps/neofetch#2019)
* Packages - Improve GUIX package detection ([dylanaraps#2021](https://github.com/dylanaraps/neofetch/pull/2021))
* Packages - Add `pm` and `cargo` (dylanaraps/neofetch#1876)
* Network - Identify network capabilities (dylanaraps/neofetch#1511)

</details>

<details>
  <summary>🖼️ Bug Fixes</summary>

* Bug Fix - Fix `col_offset` ([dylanaraps#2042](https://github.com/dylanaraps/neofetch/pull/2042))
* Bug Fix - Prioritize `/etc/os-release` ([dylanaraps#2067](https://github.com/dylanaraps/neofetch/pull/2067))
* Bug Fix - Ignore case when counting `.appimage` (dylanaraps/neofetch#2006)
* Bug Fix - Fix BSD freezing if pkg is not bootstrapped (dylanaraps/neofetch#2014)
* Bug Fix - Fix wrong icon theme (dylanaraps/neofetch#1873)

</details>

### 1.2.0

* 🚀 Take over `neofetch` with `neowofetch`

<details>
  <summary>🖼️ Ascii Art Changes</summary>

* Ascii - Add uwuntu ([#9](https://github.com/hykilpikonna/hyfetch/pull/9)) (use it with `hyfetch --test-distro uwuntu` or `neowofetch --ascii_distro uwuntu`)
* Ascii - Better Void ascii art ([#10](https://github.com/hykilpikonna/hyfetch/pull/10))
* Ascii - Update old NixOS logo for compatibility ([dylanaraps#2114](https://github.com/dylanaraps/neofetch/pull/2114))

</details>

<details>
  <summary>🖼️ Distro/OS Support Changes</summary>

* OS - Identify macOS 13 Ventura ([#8](https://github.com/hykilpikonna/hyfetch/pull/8))
* OS - Windows 11 Fluent ([dylanaraps#2109](https://github.com/dylanaraps/neofetch/pull/2109))
* Distro - Add Asahi Linux ([dylanaraps#2079](https://github.com/dylanaraps/neofetch/pull/2079))
* Distro - Add CenterOS ([dylanaraps#2097](https://github.com/dylanaraps/neofetch/pull/2097))
* Distro - Add Finnix ([dylanaraps#2099](https://github.com/dylanaraps/neofetch/pull/2099))
* Distro - Add Miracle Linux ([dylanaraps#2085](https://github.com/dylanaraps/neofetch/pull/2085))
* Distro - Add Univalent ([dylanaraps#2162](https://github.com/dylanaraps/neofetch/pull/2162))
* Distro - Add NomadBSD ([dylanaraps#2147](https://github.com/dylanaraps/neofetch/pull/2147))
* Distro - Add GrapheneOS ([dylanaraps#2146](https://github.com/dylanaraps/neofetch/pull/2146))
* Distro - Add ShastraOS ([dylanaraps#2149](https://github.com/dylanaraps/neofetch/pull/2149))
* Distro - Add Ubuntu Touch ([dylanaraps#2167](https://github.com/dylanaraps/neofetch/pull/2167))
* Distro - Add Ubuntu Sway ([dylanaraps#2136](https://github.com/dylanaraps/neofetch/pull/2136))
* Distro - Add Orchid Linux ([dylanaraps#2144](https://github.com/dylanaraps/neofetch/pull/2144))
* Distro - Add AOSC OS/Retro ([dylanaraps#2124](https://github.com/dylanaraps/neofetch/pull/2124))
* Distro - Add Ultramarine Linux ([dylanaraps#2115](https://github.com/dylanaraps/neofetch/pull/2115))
* Distro - Improve NixOS version detection ([dylanaraps#2157](https://github.com/dylanaraps/neofetch/pull/2157))

</details>

<details>
  <summary>🖼️ Device/Program Support Changes</summary>

* Terminal - Add Termux (dylanaraps/neofetch#1923)
* CPU - Add loongarch64 ([dylanaraps#2140](https://github.com/dylanaraps/neofetch/pull/2140))
* CPU - Identify CPU name for ARM / RISCV ([dylanaraps#2139](https://github.com/dylanaraps/neofetch/pull/2139))
* Battery - Fix file not found ([dylanaraps#2130](https://github.com/dylanaraps/neofetch/pull/2130))
* GPU - Identify open-kernal Nvidia driver version ([dylanaraps#2128](https://github.com/dylanaraps/neofetch/pull/2128))

</details>

<details>
  <summary>🖼️ Bug Fixes</summary>

* Bug Fix - Fix broken fedora output ([dylanaraps#2084](https://github.com/dylanaraps/neofetch/pull/2084))

</details>

<img width="200px" src="https://user-images.githubusercontent.com/22280294/181790059-47aa6f80-be99-4e67-8fa5-5c02b02842c6.png" align="right">

### 1.1.3rc1

* 🌈 Add foreground-background color arrangement to make Fedora and Ubuntu look nicer
* 🌈 Allow typing abbreviations in flag selection
* 🌈 Fix: Duplicate random color arrangements are appearing in selection screen
* 🌈 Fix: Inconsistant color arrangement when saved to config file

### 1.1.2

* Add more flags ([#5](https://github.com/hykilpikonna/hyfetch/pull/5))
* Removed `numpy` dependency that was used in 1.1.0

<img width="200px" src="https://user-images.githubusercontent.com/22280294/180901539-014f036e-c926-4470-ac72-a6d6dcf30672.png" align="right">

### 1.1.0

* Refactored a lot of things
* Added Beiyang flag xD
* Added interactive configurator for brightness adjustment
* Added dark/light mode selection
* Added color bar preview for RGB/8bit mode selection
* Added random color arrangement feature (for NixOS)

### 1.0.7

* Fix: Make config path not on init but when it's actually needed.

### 1.0.6

* Remove `hypy_utils` dependency to make packaging easier.

### 1.0.5

* Fix terminal emulator detection ([PR #2](https://github.com/hykilpikonna/hyfetch/pull/2))

### 1.0.4

* Add more flags ([PR #1](https://github.com/hykilpikonna/hyfetch/pull/1))

### 1.0.3

* Fix missing dependency for setuptools

### 1.0.2

* Implement RGB to 8bit conversion
* Add support for Python 3.7 and 3.8

### 1.0.1

* Included 11 flag presets
* Ability to lighten colors with `--c-set-l <lightness>`
* Command-line flag chooser
* Supports Python >= 3.9

## More Screenshots

![image](https://user-images.githubusercontent.com/22280294/162614578-3b878abb-2a32-4427-997e-f90b3f5cfd7c.png)
![image](https://user-images.githubusercontent.com/22280294/162661621-f1c61338-7857-4d3f-9fe3-c6b635d68c38.png)

## Original Readme from Neofetch Below

<h3 align="center"><img src="https://i.imgur.com/ZQI2EYz.png" alt="logo" height="100px"></h3>
<p align="center">A command-line system information tool written in bash 3.2+</p>

<p align="center">
<a href="./LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
<a href="https://github.com/dylanaraps/neofetch/releases"><img src="https://img.shields.io/github/release/dylanaraps/neofetch.svg"></a>
<a href="https://repology.org/metapackage/neofetch"><img src="https://repology.org/badge/tiny-repos/neofetch.svg" alt="Packaging status"></a>
</p>

<img src="https://i.imgur.com/GFmC5Ad.png" alt="neofetch" align="right" height="240px">

Neofetch is a command-line system information tool written in `bash 3.2+`. Neofetch displays information about your operating system, software and hardware in an aesthetic and visually pleasing way.

The overall purpose of Neofetch is to be used in screen-shots of your system. Neofetch shows the information other people want to see. There are other tools available for proper system statistic/diagnostics.

The information by default is displayed alongside your operating system's logo. You can further configure Neofetch to instead use an image, a custom ASCII file, your wallpaper or nothing at all.

<img src="https://i.imgur.com/lUrkQBN.png" alt="neofetch" align="right" height="240px">

You can further configure Neofetch to display exactly what you want it to. Through the use of command-line flags and the configuration file you can change existing information outputs or add your own custom ones.


Neofetch supports almost 150 different operating systems. From Linux to Windows, all the way to more obscure operating systems like Minix, AIX and Haiku. If your favourite operating system is unsupported: Open up an issue and support will be added.


### More: \[[Dependencies](https://github.com/dylanaraps/neofetch/wiki/Dependencies)\] \[[Installation](https://github.com/dylanaraps/neofetch/wiki/Installation)\] \[[Wiki](https://github.com/dylanaraps/neofetch/wiki)\]
