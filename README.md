Organic LSP
====================

## What is it?

A language server for [the Organic programming language](https://github.com/ERSUCC/Organic).  Mainly tested in Neovim.

## How do I set it up?

### Neovim

Build the project with `cargo run --release`.  Then, add the following to your Neovim config (with the path filled in):

```lua
vim.filetype.add({
  extension = {
    organic = "organic",
  },
})

vim.lsp.config("organic-lsp", {
  cmd = { "/PATH/TO/organic-lsp/target/release/organic-lsp" },
  filetypes = { "organic" },
  root_markers = { ".git" },
})

vim.lsp.enable("organic-lsp")
```

#### Debugging

To run a single instance of the server for debugging, make sure that you have the `socat` command line utility installed, launch the server with `cargo run --release`, and change the Neovim `config` section to:

```lua
vim.lsp.config("organic-lsp", {
  cmd = { "socat", "-", "TCP:127.0.0.1:9257", },
  filetypes = { "organic" },
  root_markers = { ".git" },
})
```

### VS Code

See the extension [here](https://github.com/TheBizzle/organic-lsp-vscode).

