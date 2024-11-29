---
title: "Toolproof Overview"
nav_title: "Overview"
nav_section: Root
weight: 1
---

Toolproof evaluates all `*.toolproof.yml` files it finds. Each file contains exactly one test.

Here's a simple file that tests the `mv` command:

```yml
name: mv moves files

steps:
  - I have a "start.txt" file with the content "hello world"
  - I run "mv start.txt end.txt"
  - The file "end.txt" should contain "hello"
  - snapshot: The file "end.txt"
    snapshot_content: |-
      ╎hello world
```

## Syntax

Toolproof files contain a `steps` array, where each item is a test step. The steps you write are matched to Toolproof functions.

Toolproof's syntax uses plaintext sentences with placeholders,
for example the first step in our example file above matches the Toolproof function:
```
I have a {name} file with the content {text}
```

When writing steps, you can specify the values inline using either single or double quotes:
```yaml
steps:
  - I have a "start.txt" file with the content 'hello world'
```

Alternatively, you can specify the values using keys, which can be preferred for long instructions or multiline values.
When doing this, place the step inside an object under a `step` key, and use curly braces to match the key names:
```yaml
steps:
  - step: I have a {filename} file with the content {text}
    filename: start.txt
    text: |-
      hello
      world
```

By convention, these steps are being written in yaml without quotes. They can also be wrapped in quotes when needed:
```yaml
steps:
  - "I have a 'start.txt' file with the content 'hello world'"
```

## Terminology

Steps are comprised of the following elements:

### Instructions

Instructions generally _do_ things. For example, creating a file:
```
I have a {file} with the content {text}
```

or running a command in a terminal:
```
I run {command}
```

Instructions can be a step on their own:
```yaml
steps:
  - I run "echo 'hi'"
```

### Retrievals

Retrievals get values, and are used either in assertions or snapshots. For example, getting the contents of a file:
```
The file {name}
```

or getting the contents of stdout:
```
stdout
```

Retrievals can be used for snapshots, or can be paired with an Assertion to make up a step.

### Assertions

Assertions test values. For example, testing exact matches:
```
be exactly {expected}
```

or having a value:
```
not be empty
```

To make up a step, join an Assertion to a Retrieval with `should`:
```yaml
steps:
  - The file "index.html" should not be empty
  - stdout should contain "hello"
```

### Snapshots

Any Retrieval can also drive a snapshot. To do so, place the step inside an object under a `snapshot` key:
```yaml
steps:
  - snapshot: stdout
  - snapshot: The file "index.html"
```

After running Toolproof in interactive mode and accepting changes, the file will be updated to:
```yaml
steps:
  - snapshot: stdout
    snapshot_content: |-
      ╎contents of
      ╎stdout go here
  - snapshot: The file "index.html"
    snapshot_content: |-
      ╎<body>
      ╎  <h1>Hello World</h1>
      ╎</body>
```

In future runs, Toolproof will ensure the retrieved value matches the `snapshot_content` key. Running Toolproof in
interactive mode (`-i`) will also allow you to accept the changes and update the file automatically.

### Snapshots

Any Retrieval can also drive an extract. To do so, place the step inside an object under a `extract` key, alongside an `extract_location` key:
```yaml
steps:
  - extract: stdout
    extract_location: "%toolproof_process_directory%/extracted_stdout.txt"
```

After running Toolproof, the value will be written to that file.

Toolproof never reads this file, so this step doesn't have any bearing on the success of the test.
Instead, this is intended to pull information from tests to use in other tooling.

## Test environment

Toolproof automatically runs tests in a temporary directory that is discarded at the end of a run.

Any steps that interact with files will act in this directory, and commands will run relative to this directory.

## Placeholders

Placeholders can be supplied for tests that require dynamic values. These can be supplied in a config file or on the command line:
```bash
npx toolproof --placeholders project_dir="$(pwd)" -i
```

These can be accessed inside any value:
```yaml
steps:
  - I run "%project_dir%/index.js"
  - step: stdout should contain {text}
    text: |-
      Hello world from %project_dir%
```

Toolproof provides some placeholders by default:

| placeholder                 | value                                                       |
| --------------------------- | ----------------------------------------------------------- |
| toolproof_process_directory | The working directory that you ran the Toolproof command in |
| toolproof_test_directory    | The temporary directory that the current test is running in |
