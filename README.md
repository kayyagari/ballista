# Catapult
A lean and simple launcher for Mirth Connect Admin Client.

## How To Use
1. Go to releases and download a suitable installer for your OS platform
2. Create a new connection or if you are using MirthConnect Admin Launcher then import existing connections from `<MCAL-root>/data/connections.json`
3. Select a connection from the list of connections on the left hand side
4. Adjust the `Java Home` field's value if necessary (please note that Catapult assumes JRE version 8 or higher was already installed on the local machine)
4. Click on `Open`

## Compiling

These compilation instructions are written for users not familiar with Rust and Tauri who just want to build and use Catapault.

You should generally follow the Tauri Getting started guide: https://tauri.app/v1/guides/getting-started/prerequisites

A good reference for how to run builds is the file .github/workflows/build-catapault.yml . If you can replicate the same steps the build pipeline does, then you should have good builds!

### MacOS

1. Open the project in VS Code. Let VS code install the suggested plugins.
1. Install Rust `brew install rust`
1. Run `npm install`
1. Run `npm run tauri build`
1. A DMG will be built at `./src-tauri/target/release/bundle/dmg/Catapult_0.1.0_aarch64.dmg`
1. Install the app as usual. An installation to `~/Applications` instead of `/Applications` is best for development.

### Linux

Should be very similar to MacOS.

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
