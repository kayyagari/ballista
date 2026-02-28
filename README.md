# Launcher

A lean and simple launcher for Open Integration Engine administration.

Originally forked from [Ballista](https://github.com/kayyagari/ballista) by [Kiran Ayyagari](https://github.com/kayyagari). Thank you Kiran for the original project and foundation this builds upon.

## How To Use

1. Go to releases and download a suitable installer for your OS platform
2. Create a new connection or import existing connections from `<MCAL-root>/data/connections.json`
3. Launch a connection by double-clicking the desired server, or select it and click the play button
4. Edit a connection by clicking the pencil icon on a server row
5. Adjust the `Java Home` field's value if necessary (JRE version 8 or higher must be installed)

## Features

- Dark theme UI with keyboard zoom support (Cmd/Ctrl +/-/0)
- Real-time server connectivity status
- Sort by group, name, last connected, or status
- Java console output viewer
- Jar signature verification with certificate trust management
- Cross-platform: macOS, Windows, Linux

## Compiling

Follow the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/) for your platform.

A good reference for build steps is [`.github/workflows/build-launcher.yml`](.github/workflows/build-launcher.yml).

### Quick Start

```bash
npm install
npm run tauri build
```

### Windows

Follow the openssl instructions at https://docs.rs/crate/openssl/0.9.24 using PowerShell:

```powershell
$env:OPENSSL_DIR='C:\Program Files\OpenSSL-Win64\'
$env:OPENSSL_INCLUDE_DIR='C:\Program Files\OpenSSL-Win64\include'
$env:OPENSSL_LIB_DIR='C:\Program Files\OpenSSL-Win64\lib'
$env:OPENSSL_NO_VENDOR=1
```

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
