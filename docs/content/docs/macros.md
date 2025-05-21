---
title: "Using Macros"
nav_title: "Macros"
nav_section: Root
weight: 6
---

Toolproof provides a macro system that allows you to reuse common sequences of steps across multiple tests.

## Creating Macros

Macros are defined in files with the `.toolproof.macro.yml` extension. Each macro file must contain:

1. A `macro` field with the instruction syntax
2. A `steps` array containing the test steps to execute

Here's a basic example:

```yml
# setup_web.toolproof.macro.yml
macro: I setup a web server

steps:
  - step: I have a "index.html" file with the content "<h1>Hello World</h1>"
  - step: I serve the directory "."
  - step: I have a "styles.css" file with the content "h1 { color: blue; }"
```

Toolproof will automatically load all macro files that it finds in your project. These files will not run as standalone tests.

## Using Macros

To use a macro in your test files, use the `macro` key followed by the macro instruction:

```yml
name: My Web Test

steps:
  - macro: I setup a web server
  - step: In my browser, I load "/"
  - step: The file "index.html" should contain "Hello World"
```

## Variables in Macros

Macros can accept variables that get substituted when used. Define variables using curly braces in the macro instruction:

```yml
# docker_service.toolproof.macro.yml
macro: I start a {service} service on port {port}

steps:
  - step: I have a "docker-compose.yml" file with the content {compose_content}
    compose_content: |-
      version: '3.8'
      services:
        %service%:
          image: %service%:latest
          ports:
            - "%port%:%port%"
          environment:
            - NODE_ENV=development
  - step: I run "docker-compose up -d %service%"
  - step: I run "sleep 3"
  - step: I run "curl -f http://localhost:%port%/health"
```

The macro can then be used to spin up different services:

```yml
steps:
  - macro: I start a "redis" service on port "6379"
  - macro: I start a {service} service on port {port}
    service: "postgres"
    port: "5432"
```
