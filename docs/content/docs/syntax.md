---
title: "Syntax and Terminology"
nav_title: "Syntax"
nav_section: Root
weight: 2
---

Toolproof tests are written in YAML with a syntax designed to be readable, without relying on
custom step definitions that hide complexity.

## Test File Structure

Every Toolproof test file follows this basic structure:

```yml
name: My Test Name

platforms: [windows, mac, linux]  # Optional platform limitation
type: reference                   # Optional, marks test as reference-only

steps:
  - step: I have a "config.json" file with the content "{}"
  - step: I run "npm test"
  - step: stdout should contain "All tests passed"
```

The required fields are:
- `name`: A descriptive name for your test
- `steps`: An array of test steps to execute

## Step Types

Toolproof supports six different types of steps:

1. **Instructions**: Actions that do something
2. **Retrievals with Assertions**: Getting values and checking them
3. **Snapshots**: Capturing and verifying output
4. **Extracts**: Saving output to external files
5. **References**: Including steps from other files
6. **Macros**: Reusing predefined step sequences

### 1. Instructions

Instructions perform actions like creating files or running commands:

```yml
steps:
  - step: I have a "index.html" file with the content "<h1>Hello</h1>"
  - step: I run "echo 'Hello World'"
  - step: I serve the directory "."
```

### 2. Retrievals with Assertions

Retrievals get values and Assertions check them:

```yml
steps:
  - step: The file "config.json" should contain "version"
  - step: stdout should not be empty
  - step: In my browser, the result of {js} should be exactly "Hello"
    js: return document.querySelector('h1').textContent;
```

Here, `The file "config.json"`, `stdout`, and `In my browser, the result of {js}` are Retrievals that return a value.
The Assertions are `should contain "version"`, `should not be empty`, and `should be exactly "Hello"`.

### 3. Snapshots

Snapshots capture output for future verification. A Snapshot is run on a Retrieval:

```yml
steps:
  - snapshot: The file "output.log"
  - snapshot: stdout
  - snapshot: In my browser, the result of {js}
    js: return document.body.innerHTML;
```

After running with `-i` (interactive mode), the file updates with captured content:

```yml
steps:
  - snapshot: stdout
    snapshot_content: |-
      ╎Hello World
  - snapshot: In my browser, the result of {js}
    js: return document.body.innerHTML;
    snapshot_content: |-
      ╎<h1>
      ╎  Hello World
      ╎</h1>
```

### 4. Extracts

Extracts save output to external files. An Extract is run on a Retrieval:

```yml
steps:
  - extract: stdout
    extract_location: "./output.txt"
```

### 5. References

References include steps from other files, relative to the test file:

```yml
steps:
  - ref: ./common-setup.toolproof.yml
```

### 6. Macros

Macros use predefined step sequences that you configure:

```yml
steps:
  - macro: I setup a web server with port {port}
    port: 3000
```

## Value Specification Syntax

Toolproof offers multiple ways to specify values in steps:

### Inline Values

You can specify values directly in the step using single or double quotes:

```yml
steps:
  - step: I have a "config.json" file with the content '{"version": "1.0.0"}'
```

### Object Values

For longer or more complex values, use object notation with keys:

```yml
steps:
  - step: I have a {filename} file with the content {content}
    filename: config.json
    content: |-
      {
        "version": "1.0.0",
        "name": "my-app",
        "description": "A test application"
      }
```

The placeholders in curly braces (`{filename}`, `{content}`) correspond to the keys in the step object.

### Quoted Steps

Toolproof files are plain YAML documents, so steps can also be quoted if required:

```yml
steps:
  - step: "I have a 'config.json' file with the content '{\"version\": \"1.0.0\"}'"
```

### Platform-Specific Steps

Steps or entire tests can be limited to specific platforms:

```yml
steps:
  - step: I run "ls -la"
    platforms: [mac, linux]
  - step: I run "dir"
    platforms: [windows]
```
