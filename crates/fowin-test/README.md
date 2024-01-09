# `fowin-test`
A test suite for `fowin` that uses [random testing](https://en.wikipedia.org/wiki/Random_testing) to ensure reliability and consistency among its backends.

## Rationale
The reliability of `fowin` is influenced by the operating system APIs it utilizes, which are not known for their dependability. Within these APIs lie numerous concealed bugs and intricacies capable of causing spontaneous behavior in the library. Consequently, it becomes important to rigorously assess the backends for their reliability and consistency. This testing approach enables the development of effective workarounds for platform-specific bugs, ensuring uniform and equivalent operation across various platforms. A notable advantage of this testing suite lies in its ability to evaluate all backend APIs by focusing on testing a singular high-level API.

## Control Flow
1. A random seed is generated
2. A set of random operations are generated using seed
    - For example, move window, resize window, rename, hide, etc.
    - Operations are constrained by rules (e.g. operate window after creation)
3. An expected set of properties for each window are generated following each operation
4. Execute operations in order
    - Some operations are created for `fowin` and some for `winit`
5. Pass control back-and-forth to `fowin` and `winit`, depending on operation
6. Use `fowin` to verify property correctness after each operation
7. If there are any discrepencies, error and output seed so the test can be reproduced

### Notes
* External processes will be generated to simulate a multi-application system, using `winit` to generate windows
* This suite is testing `winit` as much as it's testing `fowin`

