---
title: "Functions Glossary"
nav_title: "Functions Glossary"
nav_section: Root
weight: 6
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

Instructions:
- `In my browser, I load {url}`
- `In my browser, I evaluate {js}`
- `In my browser, I screenshot the viewport to {filepath}`
- `In my browser, I screenshot the element {selector} to {filepath}`

Retrievals:
- `In my browser, the result of {js}`
  - Returns a a value of the returned type
- `In my browser, the console`
  - Returns a string value

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
