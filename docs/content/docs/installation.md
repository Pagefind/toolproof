---
title: "Installing and running Toolproof"
nav_title: "Installing Toolproof"
nav_section: Root
weight: 5
---

Toolproof is a static binary with no dynamic dependencies, so in most cases will be simple to install and run. Toolproof is currently supported on Windows, macOS, and Linux distributions.

## Ensuring Toolproof is running a supported version

For all installation methods, your Toolproof configuration can specify the supported Toolproof versions.

```yml
# In toolproof.yml
supported_versions: ">=0.10.3"
```

This can also be set in a `TOOLPROOF_SUPPORTED_VERSIONS` environment variable.

## Running via npx

```bash
npx toolproof
```

Toolproof publishes a [wrapper package through npm](https://www.npmjs.com/package/toolproof), which is the easiest way to get started. This package will download the correct [binary of the latest release](https://github.com/CloudCannon/toolproof/releases) as an npm dependency for your platform and run it.

Specific versions can be run by passing a version tag:

```bash
npx toolproof@latest

npx toolproof@v0.1.0
```

## Downloading a precompiled binary

If you prefer to install Toolproof yourself, you can download a [precompiled release from GitHub](https://github.com/CloudCannon/flatlake/releases) and run the binary directly:

```bash
./toolproof
```

## Building from source

If you have [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed, you can run `cargo install toolproof` to build from source.

```bash
cargo install toolproof
toolproof
```
