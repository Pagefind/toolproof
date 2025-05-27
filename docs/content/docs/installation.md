---
title: "Installing and running Toolproof"
nav_title: "Installing Toolproof"
nav_section: Root
weight: 3
---

### Running via npx (Recommended)

The easiest way to get started is through npm:

```bash
npx toolproof
```

This wrapper package downloads the correct [binary](https://github.com/CloudCannon/toolproof/releases) for your platform and runs it.

You can specify versions with:

```bash
npx toolproof@latest
npx toolproof@v0.1.0
```

### Downloading a Precompiled Binary

You can download a [precompiled release from GitHub](https://github.com/CloudCannon/toolproof/releases) and run it directly:

```bash
./toolproof
```

### Building from Source

If you have [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed:

```bash
cargo install toolproof
toolproof
```

## Browser Requirements

For browser testing functionality, Toolproof requires Chrome, Chromium, or Microsoft Edge to be installed on your system. See the [Browser Testing](browser-testing/) page for detailed installation and configuration requirements.

## Basic Usage

```bash
# Run all tests
npx toolproof

# Run in interactive mode (for updating snapshots)
npx toolproof -i

# Run a specific test
npx toolproof --name "My Test Name"

# See all options
npx toolproof --help
```

## Ensuring Compatible Versions

You can specify supported Toolproof versions in your configuration:

```yml
# In toolproof.yml
supported_versions: ">=0.15.0"
```

This can also be set with the `TOOLPROOF_SUPPORTED_VERSIONS` environment variable.

## Additional Options

For a complete list of command-line options, environment variables, and configuration settings, see the [Configuration and Options](configuration/) page.
