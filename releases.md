## 0.1.0 (not yet released)

- Add documentation
- Breaking changes
  - Renamed `derive_delegate` to `fill_delegate` (9d91723)
  - Stop filling trait function with default implementation (46cc6c7)

## 0.0.3

- Prevent trait ambiguity (18ed751): Switched to generate `Hello::hello(delegatee, ...)` instead of `delegatee.hello(...)`.
- Add argument `with_uses` of `exteranl_trait_def` (9418deb)

## 0.0.2

- Prevent warnings for `unused_imports` (84c513f)

## 0.0.1

Initial release.
