Organic LSP
====================

## What is it?

A language server for [the Organic programming language](https://github.com/ERSUCC/Organic).  Mainly tested in Neovim.

## How do I set it up?

### Neovim

Build the project with `cargo build --release`.  Then, add the following to your Neovim config (with the path filled in):

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

To run a single instance of the server for debugging, make sure that you have the `socat` command line utility installed, launch the server with `cargo run debug --release`, and change the Neovim `config` section to:

```lua
vim.lsp.config("organic-lsp", {
  cmd = { "socat", "-", "TCP:127.0.0.1:9257", },
  filetypes = { "organic" },
  root_markers = { ".git" },
})
```

### VS Code

See the extension [here](https://github.com/TheBizzle/organic-lsp-vscode).

## What can it do?

  * Semantic code highlighting
  * Typechecking for nearly all of the Organic programming language
    * The only significant language feature currently missing is `include`
  * Additional information displayed on hover
  * Linting
    * Of identifiers that aren't kebab-cased
  * Navigation
    * Find all usages of a term
    * Jump to definition of term
  * Quick-fixes for
    * Non-kebab-cased names
  * Code-processing actions
    * Rename variables
    * Inline variable definitions
    * Extract the selection into a new variable
  * Code completion of
    * Built-in constants, note names, functions
    * Named parameters to built-in functions
    * User-defined variables

## What additional things might it be able to do in the future?

  * Language features
    * Import foreign terms through `include`

  * LSP features
    * Incremental (procedure-level) updates of the LSP state
    * Fuller hover documentation (in place of things that currently say `FILL_IN`)
    * Better code completion
      * Allowed immediately after whitespace
      * Disabled in strings and comments
      * Sorted by priority
      * With context/scope/type awareness
    * Formatting of code selection
    * Whole-document formatting (while preserving whitespace and comments)

