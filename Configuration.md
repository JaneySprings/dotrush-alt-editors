## Configuration
DotRush can be configured by creating a `dotrush.config.json` file in your project root directory or next to the server executable.

You only need to provide the `projectOrSolutionFiles` option if the server can't detect a project to load automaticaly. You can customize the behavior with additional settings as needed.
```json
{
    "dotrush": {
        "roslyn": {
            "projectOrSolutionFiles": [
                "/path/to/your/solution.sln"
            ]
        }
    }
}
```

All available configuration options can be found in the DotRush extension's [package.json](https://github.com/JaneySprings/DotRush/blob/main/package.json) file. Any option under the `dotrush.roslyn` namespace can be used in your settings file with the structure shown above.