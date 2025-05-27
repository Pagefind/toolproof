---
title: "Functions Glossary"
nav_title: "Functions Glossary"
nav_section: Root
weight: 7
---

Toolproof provides the following Instructions:

## Filesystem

Instructions:
- `I have a {filename} file with the content {contents}`

Retrievals:
- `The file {filename}`
  - Returns a string value

## Process

Instructions:
- `I have the environment variable {name} set to {value}`
- `I run {command}`
- `I run {command} and expect it to fail`

Retrievals:
- `stdout`
  - Returns a string value
- `stderr`
  - Returns a string value

## Hosting

Instructions:
- `I serve the directory {dir}`

## Browser

For comprehensive browser testing documentation and examples, see [Browser Testing](browser-testing/).

Instructions:
- `In my browser, I load {url}` - Navigate to a URL
- `In my browser, I evaluate {js}` - Execute JavaScript code
- `In my browser, I screenshot the viewport to {filepath}` - Capture full viewport
- `In my browser, I screenshot the element {selector} to {filepath}` - Capture specific element
- `In my browser, I click {text}` - Click element by visible text
- `In my browser, I hover {text}` - Hover over element by visible text
- `In my browser, I click the selector {selector}` - Click element by CSS selector
- `In my browser, I hover the selector {selector}` - Hover over element by CSS selector
- `In my browser, I scroll to the selector {selector}` - Scroll element into view
- `In my browser, I press the {keyname} key` - Send keyboard input (Enter, Tab, Escape, etc.)
- `In my browser, I type {text}` - Type text into focused element

Retrievals:
- `In my browser, the result of {js}` - Execute JavaScript and return the result
  - Returns a value of the returned type
- `In my browser, the console` - Get all browser console output
  - Returns a string value

### Browser Console API

When executing JavaScript in browser steps, you have access to a `toolproof` object with additional utilities:

- `await toolproof.querySelector(selector)` - Find element with timeout
- `await toolproof.querySelectorAll(selector)` - Find all elements with timeout
- `await toolproof.waitFor(() => condition)` - Wait for custom condition
- `toolproof.assert_eq(left, right)` - Assert equality
- `toolproof.assert_lte(left, right)` - Assert left ≤ right
- `toolproof.assert_gte(left, right)` - Assert left ≥ right
- `toolproof.assert(value)` - Assert value is truthy

## Assertions

### Exact assertions
- `be exactly {expected}`
- `not be exactly {expected}`

Exact assertions can compare complex objects. For example:
```yaml
steps:
  - step: In my browser, the result of {js} should be exactly {result}
    js: |-
      return { hello: "world", numbers: [1, 2, 3] };
    result:
      hello: world
      numbers:
        - 1
        - 2
        - 3
```

### Contain assertions
- `contain {expected}`
- `not contain {expected}`

### Presence assertions
- `be empty`
- `not be empty`

## Timeouts

Browser actions have a default timeout which can be configured at the command line (see `--browser-timeout` option). During this period, Toolproof will wait for elements to appear when using selectors or text interactions.

Other operations have a separate timeout that can be configured with the `--timeout` option.
