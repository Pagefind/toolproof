---
title: "Debugger Mode"
nav_title: "Debugger"
nav_section: Root
weight: 10
---

Toolproof's debugger mode allows you to run tests step-by-step, making it easier to understand test behavior, debug failures, and develop new tests.

## Enabling Debugger Mode

Run Toolproof with the `--debugger` flag along with a specific test:

```bash
# Debug a test by name
npx toolproof --debugger --name "My Test Name"

# Debug a specific test file
npx toolproof --debugger --path tests/my-test.toolproof.yml
```

Debugger mode requires running a single test. If you don't specify a test, Toolproof will show an error.

When running in debugger mode:

- The browser runs with a visible window (not headless)
- Before each step executes, Toolproof pauses and shows you:
   - The upcoming step to be executed
   - The step's arguments (if any)
   - The temporary directory path
   - The server port (if a server is running)
