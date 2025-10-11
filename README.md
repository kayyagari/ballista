# Ballista
A lean and simple launcher compatible with Open Integration Engine and Mirth Connect®.

## How To Use
1. Go to releases and download a suitable installer for your OS platform
2. Create a new connection or if you are using MirthConnect Admin Launcher then import existing connections from `<MCAL-root>/data/connections.json`

3.
    - Launch a connection by clicking on the desired server 
    - A connection can be edited by clicking on the wrench icon in the right end of the row
4. Adjust the `Java Home` field's value if necessary (please note that Ballista assumes JRE version 8 or higher was already installed on the local machine)

## Known Issues
Ballista cannot open MC Admin Client for version 3.10.1 due to a [bug in MC server](https://github.com/nextgenhealthcare/connect/issues/4432).
This bug in MC server was fixed in version 3.11.0.

## Compiling

These compilation instructions are written for users not familiar with Rust and Tauri who just want to build and use Ballista.

You should generally follow the Tauri Getting started guide: https://tauri.app/v1/guides/getting-started/prerequisites

A good reference for how to run builds is the file [`.github/workflows/build-ballista.yml`](.github/workflows/build-ballista.yml).
If you can replicate the same steps the build pipeline does, then you should have good builds!

### MacOS

1. Open the project in VS Code. Let VS code install the suggested plugins.
2. Install Rust `brew install rust`
3. Run `npm install`
4. Run `npm run tauri build`
5. A DMG will be built at `./src-tauri/target/release/bundle/dmg/Ballista_0.1.0_aarch64.dmg`
6. Install the app as usual. An installation to `~/Applications` instead of `/Applications` is best for development.

### Linux

Should be very similar to MacOS.

#### Arch Linux

Some Arch-based distros require compiling the application with `NO_STRIP=true` environment variable.

### Windows

Please make a PR if you use Windows and know how to compile the app!

___Follow the instructions at___: https://tauri.app/v1/guides/getting-started/prerequisites/#setting-up-windows

Follow the openssl instructions at: https://docs.rs/crate/openssl/0.9.24 *EXCEPT* you have to use different commands to set env vars in PowerShell:
```
$env:OPENSSL_DIR='C:\Program Files\OpenSSL-Win64\'
$env:OPENSSL_INCLUDE_DIR='C:\Program Files\OpenSSL-Win64\include'
$env:OPENSSL_LIB_DIR='C:\Program Files\OpenSSL-Win64\lib'
$env:OPENSSL_NO_VENDOR=1
Get-ChildItem Env
```
