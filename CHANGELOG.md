# Changelog

<!--
    Add changes to the Unreleased section during development.
    Do not change this header — the GitHub action that releases
    this project will edit this file and add the version header for you.
    The Unreleased block will also be used for the GitHub release notes.
-->

## Unreleased

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
