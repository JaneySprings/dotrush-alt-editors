## DotRush with Sublime Text
1. Install `LSP` plugin for Sublime Text (**Package Control: Install Package** -> **LSP**).
2. Download the latest release of the DotRush server from [GitHub Releases](https://github.com/JaneySprings/DotRush/releases).
2. Open `LSP` settings file (**Preferences** -> **Package Settings** -> **LSP** -> **Settings**).
3. Create the following configuration in the `LSP` settings file:
```json
{
	"clients": {
        "dotrush": {
            "enabled": true,
            "command": ["Your\\Path\\To\\DotRush.exe"],
            "selector": "source.cs",
        }
    }
}
```

## Configuration
See [Configuration.md](Configuration.md)
