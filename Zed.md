## DotRush with Zed
1. Install `Rust` via [rustup](https://www.rust-lang.org/tools/install).
2. Install the [.NET SDK](https://dotnet.microsoft.com/download) 10 or higher and make sure `dotnet` is available on your `PATH` (or set the `DOTNET_ROOT` environment variable).
3. Download the `Zed` folder and place it in any location on your computer.
4. Open Zed and go to the `Extensions` tab.
5. Click on the `Install Dev Extension` button.
6. Select the folder you downloaded in step 3.
7. Restart Zed.

On first start the extension downloads `DotRush.Bundle.LanguageServer.zip` from the latest [GitHub release](https://github.com/JaneySprings/DotRush/releases) into the extension's work directory and runs it with `dotnet DotRush.dll`. The bundle is a universal framework-dependent build, so there is nothing to `chmod`.

*If you see the `failed to spawn command` error message, Zed could not find the `dotnet` host. Check that it is on your `PATH` (or set `DOTNET_ROOT`) in your login shell and restart Zed.*

*To force a fresh download, delete the `bin/LanguageServer` folder in the extension's work directory and restart Zed:*
```bash
#MacOS
rm -rf "/Users/You/Library/Application Support/Zed/extensions/work/dotrush/bin/LanguageServer"

#Linux
rm -rf "/home/You/.local/share/zed/extensions/work/dotrush/bin/LanguageServer"
```

## Configuration
See [Configuration.md](Configuration.md)
