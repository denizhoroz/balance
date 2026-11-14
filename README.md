<img width="1110" height="633" alt="Adsız" src="https://github.com/user-attachments/assets/d060ee62-c6fa-40d1-9ace-86f66f6566f8" />

# Balance

A small Windows desktop app for tracking incomes, expenses and assets. Data is stored in an Excel workbook (`db\balance.xlsx`).

## Features

- Incomes and expenses with a name, value, currency and date.
- Balance (incomes minus expenses, up to today) and its change over an interval: 1D, 1W, 1M or 1Y.
- Assets recorded as dated value snapshots, grouped into categories and shown as a ring chart.
- Assets table listing each current holding with its value, change and profit over the selected interval.
- Display currencies: USD, EUR, GBP, TRY, JPY. Assets can also be held in gold or silver (XAU, XAG, in grams) or crypto (BTC, ETH).
- Amounts in other currencies are converted with exchange rates and prices fetched from the sources listed in `db\links.json`. Fetched prices are cached in `.cache\`.
- The workbook can be edited in Excel; the app reloads it when the file changes. While Excel has it open, the app cannot save and shows "unsaved" until it can.
- Light and dark theme.

## Install

Run `Balance-<version>-setup.exe`. The installer asks whether to install:

- for all users: `C:\Program Files\Balance`, requires administrator rights. All Windows users on the PC share the same data.
- for the current user only: `%LOCALAPPDATA%\Programs\Balance`, no administrator rights needed.

The install folder contains:

```
Balance\
|- .cache\        fetched prices, created by the app
|- db\            balance.xlsx, settings.toml, links.json
|- balance.exe
```

Uninstalling removes the program and `.cache\`, then asks whether to delete `db\` as well. The default is to keep it.

## Build from source

Requirements:

- Windows
- Rust with the MSVC toolchain
- Visual Studio 2022 Build Tools with "Desktop development with C++"
- CMake on `PATH`
- Inno Setup 6 (only for building the installer)

Run the app (uses `db\` and `.cache\` in the repository root):

```
cd app
cargo run
```

Run the tests:

```
cd app
cargo test
```

Build the release exe and the installer:

```
powershell -ExecutionPolicy Bypass -File installer\build.ps1
```

This writes `app\target\release\balance.exe` and `dist\Balance-<version>-setup.exe`. The version is taken from `app\Cargo.toml`.

## License

MIT.

The Inter font in `assets\fonts` is licensed under the SIL Open Font License 1.1 (see `assets\fonts\Inter-LICENSE.txt`).
