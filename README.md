# Rust for Windows Installer Custom Actions

[![latest version](https://img.shields.io/crates/v/msica?logo=rust)](https://crates.io/crates/msica)
![build status](https://github.com/heaths/msica-rs/actions/workflows/ci.yml/badge.svg?event=push)

Writing [custom actions] for [Windows Installer] (MSI) can be difficult enough already,
but using Rust can help mitigate some potential issues concerning memory and handle leaks.

These APIs roughly mimic the Windows Installer [automation interface] for those APIs
that can be called in immediate and deferred custom actions.

## Example

You can define custom actions in Rust using its [foreign function interface][ffi] like:

```rust
use msica::*;

const ERROR_SUCCESS: u32 = 0;
const ERROR_INSTALL_FAILURE: u32 = 1603;

#[no_mangle]
pub extern "C" fn MyCustomAction(session: Session) -> u32 {
    match Record::with_fields(
        Some("this is [1] [2]"),
        vec![
            Field::IntegerData(1),
            Field::StringData("example".to_owned()),
        ],
    ) {
        Ok(record) => {
            session.message(MessageType::User, &record);
            ERROR_SUCCESS
        }
        _ => ERROR_INSTALL_FAILURE,
    }
}
```

### Using nightly feature

If you enable the `nightly` feature, you can use the question mark operator (`?`) to propagate errors:

```rust
use msica::*;

#[no_mangle]
pub extern "C" fn MyCustomAction(session: Session) -> CustomActionResult {
    let record = Record::with_fields(
        Some("this is [1] [2]"),
        vec![Field::IntegerData(1), Field::StringData("example".to_owned())],
    )?;
    session.message(MessageType::User, &record);
    CustomActionResult::Succeed
}
```

## License

This project is licensed under the [MIT license](LICENSE.txt).

[automation interface]: https://docs.microsoft.com/windows/win32/msi/automation-interface
[custom actions]: https://docs.microsoft.com/windows/win32/msi/custom-actions
[ffi]: https://doc.rust-lang.org/nomicon/ffi.html
[Windows Installer]: https://docs.microsoft.com/windows/win32/msi/about-windows-installer
