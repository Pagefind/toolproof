---
title: "Reference Other Files"
nav_title: "Reference Other Files"
nav_section: Root
weight: 5
---

Toolproof allows you to reference another file and embed its steps.

To reference another file, use the `ref` key with a relative path to the target file.

If we have a file at `tests/simple.toolproof.yml` containing:
```yaml
name: Simple Test

steps:
  - step: I have a "config.js" file with the content {js}
    js: |-
      console.log("hello world");
  - stdout should contain "hello"
```

Then a file at `tests/nested/simple-plus.toolproof.yml` could contain:
```yaml
name: Simple Plus More

steps:
  - ref: ../simple.toolproof.yml
  - snapshot: stderr
```

This will embed the steps from the referenced file into this file.

If we didn't want our original `tests/simple.toolproof.yml` file to run as a standalone test, we can also tell toolproof
that this file is a reference:
```yaml
name: Simple Setup
type: reference

steps:
  - step: I have a "config.js" file with the content {js}
    js: |-
      console.log("hello world");
```

Toolproof will avoid running this file on its own, but will run the steps if they're embedded into another file.
