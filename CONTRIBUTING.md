# Contributing

Contributions are welcome, but please keep in mind the following goals of this project:

* Public APIs are only those which can be called in [custom actions]. See also:
  * [Functions Not for Use in Custom Actions](https://docs.microsoft.com/windows/win32/msi/functions-not-for-use-in-custom-actions)
  * [Obtaining Context Information for Deferred Execution Custom Actions](https://docs.microsoft.com/windows/win32/msi/obtaining-context-information-for-deferred-execution-custom-actions)
* The model for public APIs should be similar to the Windows Installer [automation interface] for some familiarity.

## Prerequisites

The following software is required:

* [Rust](https://www.rust-lang.org/tools/install)

  If Rust is already installed, please run `rustup update` to make sure you're up to date.

The following software is recommended:

* [Visual Studio Code](https://code.visualstudio.com/)

  When opening this project directory and prompted, please install recommended extensions.
  These are limited only to things that help keep the project clean and should not feel intrusive.

* [WiX 3.11](https://wixtoolset.org/releases/)

  WiX 4 is not supported at this time, but you should be able to install both side by side.

## Testing

To build and test, simply run:

```powershell
cargo test --all
```

By default, this will build x64 custom action DLLs from under _examples_.
These are used when building a Windows Installer package that will not register itself
so you can run it as many times as necessary to test custom actions:

```powershell
msbuild -t:rebuild examples/product.wixproj
msiexec /i target/debug/product.msi /l*v install.log
```

You do not need to build the MSI, but it is recommended if you make any changes
to custom action examples or the product authoring. Even if you don't,
the continuous integration (CI) build will make sure it builds cleanly.

### x86

If you have the x86 libraries for the Windows SDK installed, you can also build x86:

```powershell
rustup target install i686-pc-windows-msvc
cargo test --all --target i686-pc-windows-msvc
msbuild -t:rebuild examples/product.wixproj -p:Platform=x86
```

This should not really be necessary and is not practically useful since x86-only
Windows platforms are out of supported.
This may only be necessary if your custom actions must use x86-only APIs from third parties.

[automation interface]: https://docs.microsoft.com/windows/win32/msi/automation-interface
[custom actions]: https://docs.microsoft.com/windows/win32/msi/custom-actions
