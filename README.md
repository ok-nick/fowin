# `fowin`
A cross-platform foreign window handling library.

## FAQ
### How is this any different from `winit`?
The goal of `winit` is to manage local application windows, whether that be creation or general window management. In contrast, `fowin` focuses on managing foreign windows unbeknownst to the local application. This requires an entirely different set of APIs that aren't currently (and unsure if ever will be) supported by `winit`.
