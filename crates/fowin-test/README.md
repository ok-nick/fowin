# `fowin-test`
A test suite for `fowin` that uses [random testing](https://en.wikipedia.org/wiki/Random_testing) to ensure reliability and consistency.

## Rationale
The operating system APIs used by `fowin` are not known for their reliability. There are many hidden bugs and subtleties that can cause the library to act spontaneously. Thus, it is important to test the backends for reliability and consistency so that we may create workarounds to platform bugs and ensure each platform operates equivalently. The nice thing about this suite is that by testing the single high-level API, we are additionally testing all of the backend APIs.

## Control Flow
1. A random seed is generated
2. A set of random operations are generated using seed
    - For example, move window, resize window, rename, hide, etc.
3. Execute operations in order
    - Some operations are created for `fowin` and some for `winit`
4. Pass control back-and-forth to `fowin` and `winit`, depending on operation
5. After each operation, use `fowin` to verify property correctness
6. If there are any discrepencies, error and output seed so the test can be reproduced

### Notes
* In addition to the control flow defined above, external processes will be created to ensure reliability against running numerous applications.
* It's important to recognize that this is testing `winit` as much as it's testing `fowin`. Two birds with one stone, I guess.

