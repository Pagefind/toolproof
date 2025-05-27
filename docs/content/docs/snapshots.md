---
title: "Snapshot Testing"
nav_title: "Snapshots"
nav_section: Root
weight: 9
---

Snapshot testing in Toolproof allows you to capture the output of retrievals (like file contents, stdout, or browser elements) and verify that they match the last-known correct value.

## What are Snapshots?

Snapshots in Toolproof:
- Capture the actual output from a test step
- Store the output directly in your test file
- Automatically compare output against stored values in future runs
- Provide a diff when snapshots don't match
- Can be updated interactively when expected changes occur

Unlike traditional assertions that require you to manually specify the expected output, snapshots let you verify that your application's output hasn't changed unexpectedly.

## Creating a Snapshot

Any retrieval in Toolproof can be turned into a snapshot. To create a snapshot, use the `snapshot` key instead of the `step` key:

```yml
steps:
  - snapshot: stdout
  - snapshot: The file "index.html"
  - snapshot: In my browser, the result of {js}
    js: return document.querySelector('h1').innerText;
```

When you first run a test with snapshots in interactive mode (`-i`), Toolproof will capture the output and prompt you to accept it.

After accepting, the snapshot is stored directly in your test file:

```yml
steps:
  - snapshot: stdout
    snapshot_content: |-
      ╎Hello, world!
      ╎This is a test.
  - snapshot: The file "index.html"
    snapshot_content: |-
      ╎<html>
      ╎  <body>
      ╎    <h1>Hello world</h1>
      ╎  </body>
      ╎</html>
  - snapshot: In my browser, the result of {js}
    js: return document.querySelector('h1').innerText;
    snapshot_content: |-
      ╎Hello world
```

## Snapshot Verification

In subsequent test runs, Toolproof will:
1. Execute the retrieval step
2. Compare the result against the stored `snapshot_content`
3. Pass the test if they match
4. Fail the test and show a diff if they don't match

## Updating Snapshots

When your application changes intentionally, you'll need to update your snapshots. To update snapshots:

1. Run Toolproof in interactive mode with the `-i` flag: `npx toolproof -i`
2. When a snapshot mismatch occurs, Toolproof will show the diff and prompt you
3. Press `y` to accept the new snapshot or `N` to reject it and fail the test

## Limitations

- Snapshots work best with deterministic outputs. Content with timestamps, random IDs, or other dynamic elements may cause tests to fail unnecessarily.

- Large snapshots can make test files harder to read. Consider using multiple focused snapshots instead of one large snapshot.

- Binary content (like images) cannot be directly snapshotted in the test file.

## Relationship with Extracts

While snapshots are used for verification, the extract feature is used to save output to a file without verification:

```yml
steps:
  - extract: stdout
    extract_location: "output.txt"
```

Unlike snapshots, extracts don't validate content but are useful for debugging or integrating with other tools.
