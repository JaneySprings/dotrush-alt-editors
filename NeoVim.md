## DotRush with Neovim
1. Ensure you have a plugin manager like [vimplug](https://github.com/junegunn/vim-plug) installed.
2. Download the `DotRush.Server` executable from the latest [GitHub release](https://github.com/JaneySprings/DotRush/releases/latest) and place it in any location on your computer.
3. Add the following lines to your init.vim configuration:

```lua
call plug#begin()
Plug 'https://github.com/neovim/nvim-lspconfig'
call plug#end()

lua << EOF
require('lspconfig.configs').dotrush = {
    default_config = {
        cmd = { '/Path/To/Executale/dotrush' }, -- Adjust path to the DotRush executable
        filetypes = { 'cs', 'xaml' },
        root_dir = function(fname)
            return vim.fn.getcwd()
        end
    };
}
require('lspconfig').dotrush.setup({})
EOF
```

## Configuration
See [Configuration.md](Configuration.md)