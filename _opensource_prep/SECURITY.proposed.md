# Security Policy

MatRust Studio is alpha software. Security reports are welcome, especially for the archive, XML, FFI, and sandbox boundaries.

## Report a vulnerability privately

Please open a [private GitHub Security Advisory](https://github.com/varunkumar898/MatRust-Studio/security/advisories/new). Do not publish an undisclosed vulnerability in a public issue or discussion.

Include the affected version or commit, operating system, impact, and a minimal reproduction that contains no real proprietary `.mlapp` or `.slx` files and no private data. Do not include an exploit against a third-party system.

## Focus areas

- ZIP/OPC extraction, path handling, decompression limits, and XML parsing.
- C ABI pointer handling, ownership, and panic boundaries.
- Runtime sandbox policies, cancellation, and external bridge boundaries.

## Supported versions

The current policy supports the latest commit on `main`. Security fixes are not guaranteed for older commits or alpha builds.

## Disclosure

The maintainer will acknowledge reports and coordinate a fix and disclosure timeline with the reporter. Please avoid public details until users have had a reasonable opportunity to update.
