# Screenshots for the README

The README gallery should use a small number of current, representative
captures. Existing candidate images in this checkout are:

- `docs/screenshots/gui-clean/after/desktop_1280x720.png` — current desktop
  layout candidate.
- `docs/screenshots/sample_app_autotest.png` — automated sample-app capture.
- `docs/assets/gui_window_sample.png` — static component layout.

Confirm these are the images you want to publish before linking them from the
README. Do not add screenshot files solely to fill a gallery slot.

## Naming and image guidance

- Use descriptive, lowercase names such as `desktop-editor.png`,
  `app-inspector.png`, and `simulation-view.png`.
- Prefer PNG for screenshots. Keep the image readable at the GitHub README
  display width and avoid unnecessarily large source images; resize copies
  rather than changing test evidence.
- Put curated README screenshots in `docs/screenshots/` or `docs/assets/`.
  Keep test-generated snapshots and historical captures in their existing
  evidence locations.
- Add concise alt text describing the visible application area.

## Privacy and release checklist

- Hide usernames, local filesystem paths, window titles containing personal
  information, email addresses, tokens, and notifications.
- Close private files and remove customer or proprietary app/model contents.
- Verify that the capture contains only synthetic project examples or
  material you have permission to publish.
- Inspect the final image at full size before committing.
- Do not edit or replace screenshots used as test evidence without updating
  the relevant test and expected image.
