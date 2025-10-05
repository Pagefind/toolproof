---
title: "Configuration and Options"
nav_title: "Configuration"
nav_section: Root
weight: 4
---

Toolproof can be configured through a configuration file or via command-line options and environment variables.

## Configuration File

You can configure Toolproof using a `toolproof.yml`, `toolproof.yaml`, `toolproof.json`, or `toolproof.toml` file in your project root. This allows you to set default options like test timeouts, browser selection, and before-test hooks.

Example configuration:

```yml
# toolproof.yml
concurrency: 10
timeout: 15
browser_timeout: 10
retry_count: 1
browser: chrome
placeholder_delimiter: "%"
placeholders:
  project_dir: "/path/to/project"
  api_key: "12345"
before_all:
  - command: "npm install"
  - command: "npm run build"
supported_versions: ">=0.15.0"
failure_screenshot_location: "./test-failures"
```

### Configuration File Options

All configuration options that can be set via command-line or environment variables can also be configured in the configuration file:

| Key | Type | Description |
|-----|------|-------------|
| `root` | String | The location from which to look for toolproof test files |
| `verbose` | Boolean | Print verbose logging while running tests |
| `porcelain` | Boolean | Reduce logging to be stable (machine-readable output) |
| `interactive` | Boolean | Run toolproof in interactive mode |
| `all` | Boolean | Run all tests when in interactive mode |
| `name` | String | Exact name of a test to run (case-sensitive) |
| `path` | String | Path to a test file or directory to run |
| `browser` | String | Specify which browser to use (`chrome` or `pagebrowse`) |
| `concurrency` | Number | How many tests should be run concurrently |
| `timeout` | Number | How long in seconds until a step times out |
| `browser_timeout` | Number | How long in seconds until actions in a browser time out |
| `placeholder_delimiter` | String | Character that delimits placeholders in test steps |
| `placeholders` | Object | Key-value pairs for placeholder replacement |
| `before_all` | Array | Commands to run before starting tests (objects with `command` key) |
| `skip_hooks` | Boolean | Skip running any before_all hooks |
| `supported_versions` | String | Error if Toolproof version doesn't match this range |
| `failure_screenshot_location` | String | Directory to save browser screenshots when tests fail |
| `retry_count` | Number | Number of times to retry failed tests before marking as failed |
| `debugger` | Boolean | Run in debugger mode with step-by-step execution (requires single test) |

## Command Line Options

Toolproof offers several command-line options to customize its behavior:

```bash
# Basic usage
npx toolproof

# Run in interactive mode
npx toolproof -i

# Specify test directory
npx toolproof --root ./tests

# Run a specific test by name
npx toolproof --name "My Test Name"

# Run a specific test file
npx toolproof --path tests/my-test.toolproof.yml

# Run all tests in a directory
npx toolproof --path tests/integration

# Provide placeholders
npx toolproof --placeholders project_dir="$(pwd)" api_key=$API_KEY

# Run with higher concurrency
npx toolproof -c 20
```

### Available Options

| Option | Description |
|--------|-------------|
| `-r, --root <DIR>` | The location from which to look for toolproof test files |
| `-c, --concurrency <NUM>` | How many tests should be run concurrently |
| `--placeholders <PAIRS>` | Define placeholders for tests (format: key=value) |
| `--placeholder-delimiter <DELIM>` | Define which character delimits placeholders (default: %) |
| `-v, --verbose` | Print verbose logging while running tests |
| `--porcelain` | Reduce logging to be stable (machine-readable output) |
| `-i, --interactive` | Run toolproof in interactive mode |
| `-a, --all` | Run all tests when in interactive mode |
| `-s, --skiphooks` | Skip running any hooks (e.g. before_all) |
| `--timeout <NUM>` | How long in seconds until a step times out |
| `--browser-timeout <NUM>` | How long in seconds until actions in a browser time out |
| `-n, --name <NAME>` | Exact name of a test to run |
| `-p, --path <PATH>` | Path to a test file or directory to run |
| `--browser <IMPL>` | Specify which browser to use for tests (chrome or pagebrowse, default: chrome) |
| `--retry-count <COUNT>` | Number of times to retry failed tests before marking them as failed |
| `--failure-screenshot-location <DIR>` | If set, Toolproof will screenshot the browser to this location when a test fails |
| `--debugger` | Run in debugger mode with step-by-step execution (requires single test with --name) |

## Environment Variables

Most options can also be set using environment variables:

| Environment Variable | Description |
|---------------------|-------------|
| `TOOLPROOF_ROOT` | The location from which to look for toolproof test files |
| `TOOLPROOF_VERBOSE` | Print verbose logging while running tests |
| `TOOLPROOF_PORCELAIN` | Reduce logging to be stable |
| `TOOLPROOF_RUN_NAME` | Run a specific test by name |
| `TOOLPROOF_RUN_PATH` | Path to a test file or directory to run |
| `TOOLPROOF_BROWSER` | Specify which browser to use (chrome or pagebrowse) |
| `TOOLPROOF_CONCURRENCY` | How many tests should be run concurrently |
| `TOOLPROOF_TIMEOUT` | How long in seconds until a step times out |
| `TOOLPROOF_BROWSER_TIMEOUT` | How long in seconds until actions in a browser time out |
| `TOOLPROOF_PLACEHOLDER_DELIM` | What delimiter should be used when replacing placeholders |
| `TOOLPROOF_SKIPHOOKS` | Skip running any of the before_all hooks |
| `TOOLPROOF_SUPPORTED_VERSIONS` | Error if Toolproof does not match this version range |
| `TOOLPROOF_FAILURE_SCREENSHOT_LOCATION` | Location for browser screenshots on test failure |
| `TOOLPROOF_RETRY_COUNT` | Number of times to retry failed tests |
| `TOOLPROOF_DEBUGGER` | Run in debugger mode with step-by-step execution |
