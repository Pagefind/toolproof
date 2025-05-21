---
title: "Using Placeholders"
nav_title: "Placeholders"
nav_section: Root
weight: 10
---

Placeholders allow you to use dynamic values in your Toolproof tests. They provide a way to make tests more flexible and to avoid hardcoded values.

## What are Placeholders?

Placeholders are special markers in your test files that get replaced with actual values at runtime. By default, placeholders are enclosed in `%` characters, like `%placeholder_name%`.

## Defining Placeholders

You can define placeholders in three ways:

### 1. In a Configuration File

```yml
# toolproof.yml
placeholders:
  api_key: "1234567890"
  base_url: "https://api.example.com"
  output_dir: "./output"
```

### 2. On the Command Line

```bash
npx toolproof --placeholders api_key="1234567890" base_url="https://api.example.com"
```

### 3. Through Environment Variables

```bash
export TOOLPROOF_PLACEHOLDERS="api_key=1234567890,base_url=https://api.example.com"
npx toolproof
```

## Using Placeholders in Tests

Placeholders can be used in any string value in your test files:

```yml
steps:
  - step: I have a "config.json" file with the content {content}
    content: |-
      {
        "apiKey": "%api_key%",
        "baseUrl": "%base_url%"
      }
  - step: "I run 'curl -H \"Authorization: Bearer %api_key%\" %base_url%/users'"
```

## Default Placeholders

Toolproof provides several built-in placeholders that are always available:

| Placeholder | Description |
|-------------|-------------|
| `toolproof_process_directory` | The working directory where you ran the Toolproof command |
| `toolproof_process_directory_unix` | Same as above, but with forward slashes for cross-platform compatibility |
| `toolproof_test_directory` | The temporary directory where the current test is running |
| `toolproof_test_directory_unix` | Same as above, but with forward slashes |
| `toolproof_test_port` | The port that Toolproof is using for serving files in this test |

These placeholders are especially useful for file paths and URLs:

```yml
steps:
  - step: I run "node %toolproof_process_directory_unix%/scripts/setup.js"
  - step: I have the environment variable "PROJECT_DIR" set to "%toolproof_process_directory%"
  - step: I serve the directory "."
  - step: I run "curl http://localhost:%toolproof_test_port%/"
```

## Customizing the Placeholder Delimiter

If you need to use a different character than `%` to delimit your placeholders, you can configure it:

```yml
# toolproof.yml
placeholder_delimiter: "$"
```

Or on the command line:

```bash
npx toolproof --placeholder-delimiter "$"
```

With this configuration, you would use placeholders like `$api_key$` instead of `%api_key%`.
