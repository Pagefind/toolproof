---
title: "Platform-Specific Tests"
nav_title: "Platforms"
nav_section: Root
weight: 8
---

Toolproof allows you to specify which platforms a test or step should run on. This is useful when you need to test platform-specific behavior or when certain steps need to be different across operating systems.

## Supported Platforms

Toolproof supports three platform specifiers:
- `windows`
- `mac`
- `linux`

## Test-Level Platform Specification

You can specify which platforms a test should run on by adding a `platforms` array to your test file:

```yml
name: Windows-specific test
platforms: [windows]

steps:
  - step: I run "dir"
  - step: stdout should contain "Directory"
```

Multiple platforms can be specified:

```yml
name: Unix-like systems test
platforms: [mac, linux]

steps:
  - step: I run "ls -la"
  - step: stdout should contain "total"
```

## Step-Level Platform Specification

Individual steps can also be platform-specific. This is particularly useful when you need different commands or paths for different operating systems:

```yml
name: Cross-platform path test

steps:
  - step: I have the environment variable "APP_PATH" set to "/usr/local/bin"
    platforms: [mac, linux]
  - step: I have the environment variable "APP_PATH" set to "C:\Program Files"
    platforms: [windows]
  - step: I run "echo %APP_PATH%"
    platforms: [windows]
  - step: I run "echo $APP_PATH"
    platforms: [mac, linux]
```

Platform specifications work with all step types:

```yml
steps:
  # Regular steps
  - step: I run "dir"
    platforms: [windows]

  # References
  - ref: ./windows_setup.toolproof.yml
    platforms: [windows]

  # Macros
  - macro: I setup Windows paths
    platforms: [windows]

  # Snapshots
  - snapshot: The file "paths.txt"
    platforms: [windows]

  # Extracts
  - extract: stdout
    extract_location: "output.txt"
    platforms: [windows]
```

## Combining Test and Step Platforms

When both test-level and step-level platform specifications are present, they work together:

1. If a test has no platform specification, it runs on all platforms
2. If a test specifies platforms, it only runs on those platforms
3. Within a test, steps without platform specifications run on all platforms the test runs on
4. Steps with platform specifications only run on the intersection of their platforms and the test's platforms

For example:

```yml
name: Mixed platform test
platforms: [windows, mac]  # Test only runs on Windows and macOS

steps:
  - step: I run "echo 'Hi'"  # Runs on both Windows and macOS

  - step: I run "dir"
    platforms: [windows]     # Only runs on Windows

  - step: I run "ls"
    platforms: [linux]       # Never runs (not in test platforms)
```
