---
title: "Getting Started with Toolproof"
nav_title: "Quick Start"
nav_section: Root
weight: 1
---

Toolproof runs after your application build, testing how users would interact with your software. With Toolproof, you write tests in a natural(ish) language syntax that's both easy to read and powerful.

Since Toolproof works with your built application, we'll start by creating a simple test file and then run Toolproof against it.

## Creating Your First Test

Create a file named `hello.toolproof.yml` with the following content:

```yml
name: Hello World Test

steps:
  - step: I have a "hello.txt" file with the content "Hello, World!"
  - step: I run "cat hello.txt"
  - step: stdout should contain "Hello"
```

This test should describe itself pretty well, and be readable by someone with a minimum of technical knowledge.

## Running Toolproof

Run Toolproof using npx:

```bash
npx toolproof
```

Toolproof will find all `.toolproof.yml` files in your project and run them. You should see output indicating that your test has passed.

## Test Environment

Toolproof runs each test in a temporary directory that is discarded after the test completes. All file operations and commands run relative to this directory unless absolute paths are used.

## Adding Snapshots

Let's enhance our test with a snapshot. Update `hello.toolproof.yml`:

```yml
name: Hello World Test

steps:
  - step: I have a "hello.txt" file with the content "Hello, World!"
  - step: I run "cat hello.txt"
  - step: stdout should contain "Hello"
  - snapshot: stdout
```

Run Toolproof in interactive mode to capture the snapshot:

```bash
npx toolproof -i
```

When prompted to accept the snapshot, type `y`. Toolproof will update your test file with the captured snapshot:

```yml
name: Hello World Test

steps:
  - step: I have a "hello.txt" file with the content "Hello, World!"
  - step: I run "cat hello.txt"
  - step: stdout should contain "Hello"
  - snapshot: stdout
    snapshot_content: |-
      ╎Hello, World!
```

In future test runs, Toolproof will verify that the output exactly matches the snapshot.

## Testing a Web Application

Toolproof can also test web applications. Create a file named `web.toolproof.yml`:

```yml
name: Web Test

steps:
  - step: I have a "index.html" file with the content {html}
    html: |-
      <html>
        <head>
          <title>Test Page</title>
        </head>
        <body>
          <h1>Hello World</h1>
          <button id="btn">Click Me</button>
          <p id="result"></p>
          <script>
            document.querySelector('#btn').addEventListener('click', function() {
              document.querySelector('#result').textContent = 'Button clicked!';
            });
          </script>
        </body>
      </html>
  - step: I serve the directory "."
  - step: In my browser, I load "/"
  - step: In my browser, I click the selector "#btn"
  - step: In my browser, the result of {js} should be exactly "Button clicked!"
    js: return await toolproof.querySelector('#result').textContent;
```

Run this test with:

```bash
npx toolproof
```

## Next Steps

- [Syntax and Terminology](syntax/): Learn the full syntax for writing tests
- [Browser Testing](browser-testing/): Comprehensive guide to testing web applications
- [Debugger Mode](debugger/): Step through tests interactively for debugging and development
- [Using Macros](macros/): Create reusable step sequences
- [Snapshot Testing](snapshots/): Snapshot test long or complex output
- [Configuration](configuration/): Configure Toolproof for your project
- [Platform-Specific Testing](platforms/): Write tests that work across operating systems

For a complete list of available functions, see the [Functions Glossary](functions/).
