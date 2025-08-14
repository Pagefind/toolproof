# Changelog

<!--
    Add changes to the Unreleased section during development.
    Do not change this header — the GitHub action that releases
    this project will edit this file and add the version header for you.
    The Unreleased block will also be used for the GitHub release notes.
-->

## Unreleased

## v0.15.1 (August 14, 2025)

* Fixed Chrome browser windows not being closed after a test run
* Updated dependencies
  * Updated wax to 0.6.0
  * Updated async-recursion to 1.1.1
  * Updated similar to 2.7.0
  * Updated inventory to 0.3.20
  * Updated tempfile to 3.20.0
  * Updated console to 0.16
  * Updated async-trait to 0.1.88
  * Updated schematic to 0.18.12
  * Updated strip-ansi-escapes to 0.2.1
  * Updated semver to 1.0.26

## v0.15.0 (May 16, 2025)

* Added a `--retry-count` option to retry failed tests

## v0.14.0 (May 14, 2025)

* Added placeholders for `toolproof_process_directory_unix` and `toolproof_test_directory_unix`
* Improved error logging when a file fails to parse

## v0.13.0 (March 25, 2025)

* Added browser instruction: `In my browser, I scroll to the selector {selector}`
  * NB: Click and hover steps already handled this, so this new instruction is primarily useful for screenshots.

## v0.12.0 (March 21, 2025)

* Added a `toolproof_test_port` default placeholder.

## v0.11.2 (March 20, 2025)

* If the `I click {text}` action finds multiple options, but only one is an exact match, it will now click it rather than error.

## v0.11.1 (March 20, 2025)

* Added support for newline and tab characters in the `I type {text}` instruction

## v0.11.0 (February 13, 2025)

* Added browser instruction: `In my browser, I press the {keyname} key`
* Added browser instruction: `In my browser, I type {text}`

## v0.10.4 (February 12, 2025)

* Fix the "I click" action when the provided text contains an apostrophe/single quote.

## v0.10.3 (January 24, 2025)

* Allow the generic "I click" action to click `option` elements, and elements with a `role="option"` attribute
* Add a `supported_versions` configuration option to ensure Toolproof isn't running a version older than your tests support
* Add a `failure_screenshot_location` configuration option to enable Toolproof to automatically screenshot the browser on test failure

## v0.10.2 (December 18, 2024)

* Allow the generic "I click" action to click elements with a `role="button"` attribute

## v0.10.1 (December 12, 2024)

* Made the browser click/hover steps more resilient to DOM nodes detaching mid-action

## v0.10.0 (December 12, 2024)

* Add `browser-timeout` / `browser_timeout` setting that changes the default timeout for browser actions such as `toolproof.querySelector()`

## v0.9.0 (December 3, 2024)

* Add automatic wait-and-timeout to Toolproof actions that get elements

## v0.8.0 (December 3, 2024)

* Add instructions for clicking and hovering elements on a page
* Added a timeout to all test steps

## v0.7.0 (November 29, 2024)

* Add screenshot instructions to Toolproof
* Add `extract` concept to pull retrievals to disk

## v0.6.1 (November 28, 2024)

* Log inner macro steps

## v0.6.0 (November 28, 2024)

* Added macro feature to Toolproof

## v0.5.0 (November 28, 2024)

* Add `before_all` commands to the Toolproof config

## v0.4.1 (October 2, 2024)

* Improve resilience launching a Chrome instance

## v0.4.0 (September 27, 2024)

* Adds `-n <name>` / `--name <name>` arguments to the CLI to run a specific test

## v0.3.0 (August 16, 2024)

* Adds platform flags for tests and individual steps
* Adds default placeholders for directories

## v0.2.0 (May 21, 2024)

* Base toolproof release
