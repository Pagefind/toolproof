---
title: Toolproof
nav_title: Home
weight: 1
---

Toolproof is a testing framework designed for CLI tools and web applications, allowing you to write tests in a natural(ish) language syntax that is both readable and powerful.

Toolproof runs external to your application, without mocks or harnesses. Ideally, Toolproof interacts with your codebase the way your users will.

## Features

- **Natural(ish) language syntax** - Tests that read like user instructions
- **Snapshot testing** - Capture and verify complex outputs without manual assertions
- **Cross-platform testing** - Run different steps based on the operating system
- **Macros** - Create reusable step sequences to reduce duplication
- **Browser automation** - Test web applications with browser interactions
- **Browser screenshots** - Capture visual state for verification, or documentation
- **File manipulation** - Create, modify, and verify files as part of tests
- **Command execution** - Run commands and verify their output
- **Web server support** - Test applications with local servers

## Quick Example

```yml
name: Test a simple web app

steps:
  - step: I have a "index.html" file with the content "<h1>Hello World</h1>"
  - step: I serve the directory "."
  - step: In my browser, I load "/"
  - step: In my browser, the result of {js} should be exactly "Hello World"
    js: return await toolproof.querySelector('h1').textContent
  - step: In my browser, I screenshot the viewport to "screenshot.png"
```

The goal of Toolproof is to make testing as natural as explaining how an application should work. Tests should be readable by developers not familiar with your project, while still being powerful enough for complex testing scenarios.

## Getting Started

Install and run Toolproof with npx:

```bash
npx toolproof
```

Or install the binary directly from [GitHub releases](https://github.com/pagefind/toolproof/releases).

See the [Quick Start Guide](docs/) for more information on setting up your first test.
