---
title: "Browser Testing"
nav_title: "Browser Testing"
nav_section: Root
weight: 11
---

Toolproof provides powerful browser automation capabilities for testing web applications. You can interact with web pages, execute JavaScript, take screenshots, and verify browser state.

## Browser Requirements

Toolproof requires Chrome, Chromium, or Microsoft Edge to be installed on your system for browser testing functionality. Toolproof uses the Chrome DevTools Protocol via [chromiumoxide](https://github.com/mattsse/chromiumoxide) to control the browser.

### Automatic Detection

Toolproof automatically detects browser installations using the following detection order:

1. **Environment Variable**: Checks the `CHROME` environment variable first
2. **PATH Search**: Searches for these executables in your system PATH:
   - `chrome`
   - `chrome-browser`
   - `google-chrome-stable`
   - `chromium`
   - `chromium-browser`
   - `msedge`
   - `microsoft-edge`
   - `microsoft-edge-stable`
3. **Registry** (Windows only): Checks Windows registry for Chrome installation
4. **Known Installation Paths**: Checks platform-specific installation directories

#### Platform-Specific Paths

**macOS**:
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- `/Applications/Chromium.app/Contents/MacOS/Chromium`
- `/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge`

**Linux**:
- `/opt/chromium.org/chromium`
- `/opt/google/chrome`

**Windows**:
- Registry entries for Chrome
- `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`

### Manual Configuration

If Toolproof cannot auto-detect your browser installation, or you want to use a specific version, set the `CHROME` environment variable:

```bash
export CHROME="/path/to/your/browser-executable"
npx toolproof
```

This works for any Chromium-based browser.

### Headless Chrome

For CI environments or headless testing, you can use Chrome's headless mode or download precompiled headless Chrome binaries from the [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/) infrastructure.

### Troubleshooting

**macOS Users**: You may need to allow browser executables through Gatekeeper when using downloaded binaries. Go to System Settings > Privacy & Security and click "Open Anyway" after the first blocked execution attempt.

## Basic Browser Operations

### Loading Pages

Start browser testing by serving content and loading a page:

```yml
steps:
  - step: I have a "index.html" file with the content "<h1>Hello World</h1>"
  - step: I serve the directory "."
  - step: In my browser, I load "/"
```

You can load any URL, including external sites:

```yml
steps:
  - step: In my browser, I load "https://example.com"
  - step: In my browser, I load "/about"
  - step: In my browser, I load "http://localhost:3000/api/status"
```

When loading a non-fully-qualified URL, such as `/about`, Toolproof will load
this page relative to the local site it is hosting from the `I serve the directory` step.

### Interacting with Elements

Toolproof provides several ways to interact with page elements:

```yml
steps:
  # Interact with visible text in a clickable element (e.g. <button>)
  - step: In my browser, I click "Submit"
  - step: In my browser, I click "Sign Up"
  - step: In my browser, I hover "Menu"

  # Interact with CSS selector
  - step: In my browser, I click the selector "#submit-btn"
  - step: In my browser, I click the selector ".primary-button"
  - step: In my browser, I hover the selector ".dropdown-trigger"

  # Scroll to elements
  - step: In my browser, I scroll to the selector "#footer"
```

### Keyboard Input

Type text and send key presses:

```yml
steps:
  # Type text (usually into focused input fields)
  - step: In my browser, I type "john@example.com"
  - step: In my browser, I type "my password"

  # Press specific keys
  - step: In my browser, I press the "Enter" key
  - step: In my browser, I press the "Tab" key
  - step: In my browser, I press the "Escape" key
```

### Taking Screenshots

Capture visual state for verification or documentation:

```yml
steps:
  # Screenshot the entire viewport
  - step: In my browser, I screenshot the viewport to "homepage.png"

  # Screenshot a specific element
  - step: In my browser, I screenshot the element "#main-content" to "content.png"
  - step: In my browser, I screenshot the element ".modal" to "modal-dialog.png"
```

Toolproof doesn't include visual snapshot diffs, so these screenshots
should be plugged into another tool if needed for regression testing.

## JavaScript Execution

### Running JavaScript Code

Execute custom JavaScript in the browser context:

```yml
steps:
  - step: In my browser, I evaluate {js}
    js: |-
      const status = await toolproof.querySelector('#status');
      status.textContent = 'Ready';
      localStorage.setItem('user', 'test@example.com');
```

### Retrieving Values with JavaScript

Get values from the page using JavaScript expressions:

```yml
steps:
  - step: In my browser, the result of {js} should be exactly "Welcome"
    js: |-
      const el = await toolproof.querySelector('h1');
      return el.textContent;

  - step: In my browser, the result of {js} should contain "success"
    js: |-
      const el = await toolproof.querySelector('.message');
      return el.innerText;

  # Complex data retrieval
  - step: In my browser, the result of {js} should be exactly {expected}
    js: |-
      return {
        title: document.title,
        url: window.location.href,
        ready: document.readyState === 'complete'
      };
    expected:
      title: "My App"
      url: "http://localhost:%toolproof_test_port%/"
      ready: true
```

Note that values returned from the browser JavaScript execution are converted automatically. This allows a JSON object returned from the browser execution to be compared
with a YAML object in the test file.

## The Browser Console API

When executing JavaScript in Toolproof, you have access to a `toolproof` object that provides additional testing utilities.

### Element Selection with Timeouts

The `toolproof` object provides enhanced element selection with automatic waiting:

```yml
steps:
  - step: In my browser, the result of {js} should be exactly "Click me"
    js: |-
      const button = await toolproof.querySelector('#my-button');
      return button.textContent;
```

These methods automatically wait for elements to appear in the DOM, with a configurable timeout (see the `--browser-timeout` option).

### Waiting for Conditions

Use `waitFor` to wait for custom conditions:

```yml
steps:
  - step: In my browser, I evaluate {js}
    js: |-
      // Wait for an API call to complete
      await toolproof.waitFor(() => {
        return window.apiCallComplete === true;
      });

      // Wait for element to have specific content
      await toolproof.waitFor(() => {
        const el = document.querySelector('#status');
        return el && el.textContent === 'Loaded';
      });
```

### Assertions in JavaScript

The `toolproof` object provides assertion methods:

```yml
steps:
  - step: In my browser, I evaluate {js}
    js: |-
      const count = document.querySelectorAll('.item').length;
      toolproof.assert_eq(count, 5);

      const progress = parseInt(document.querySelector('#progress').value);
      toolproof.assert_gte(progress, 0);
      toolproof.assert_lte(progress, 100);

      const modal = await toolproof.querySelector('#modal');
      const isVisible = modal.style.display !== 'none';
      toolproof.assert(isVisible);
```

Available assertion methods:
- `toolproof.assert_eq(left, right)` - Assert equality
- `toolproof.assert_lte(left, right)` - Assert left ≤ right
- `toolproof.assert_gte(left, right)` - Assert left ≥ right
- `toolproof.assert(value)` - Assert value is truthy

## Console Output

Capture browser console output for debugging:

```yml
steps:
  - step: In my browser, I load "/"
  - step: In my browser, the console should be empty
  - step: In my browser, I evaluate "console.log('Test message')"
  - step: In my browser, the console should not be empty
  - snapshot: In my browser, the console
    snapshot_content: |-
      ╎- 'LOG: Test message'
      ╎- 'ERR: Test error'
```

The console retrieval captures all console output including `console.log`, `console.warn`, `console.error`, and `console.debug` messages.

## Error Handling

JavaScript errors in the browser are automatically captured and reported:

```yml
steps:
  - step: In my browser, I evaluate {js}
    js: |-
      // This will cause a test failure due to the error
      throw new Error("Something went wrong");
```

## Browser Configuration

Configure browser behavior through command-line options or configuration file:

```bash
# Set browser timeout to 30 seconds
npx toolproof --browser-timeout 30
```

In `toolproof.yml`:

```yml
browser: chrome
browser_timeout: 30
failure_screenshot_location: "./test-failures"
```

When browser tests fail, Toolproof can automatically capture screenshots to help with debugging. Set `failure_screenshot_location` to enable this feature.
