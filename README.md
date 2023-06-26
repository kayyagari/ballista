# Catapult
A lean and simple launcher for Mirth Connect Admin Client.

## How To Use
1. Open the application
2. If not already done then import `<MCAL-root>/data/connections.json`
3. Select a connection from the list of connections on the left hand side
4. Click on `Open`

## Compiling

These compilation instructions are written for users not familiar with Rust and Tauri who just want to build and use Catapault.

You should generally follow the Tauri Getting started guide: https://tauri.app/v1/guides/getting-started/prerequisites

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

Please make a PR if you use Windows and know how to compile the app