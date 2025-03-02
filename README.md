### Stacky

If panic happened, stacky will send the backtrace to all instances of neovim it finds that 
match the CWD of any of the backtrace entries.
From there you can open the location from the list.

### Usage
You need to have a stacky lua plugin installed in Neovim to use this binary.

```bash
RUST_BACKTRACE=full yourprogram 2>&1 | stacky
```

# TODO:
* If the new neovim instance opens, send the backtrace
* Panic handler override that does the same without piping (but needs a lib in source)
* ~~Somehow pick the correct Neovim instance to notify.~~ Done by finding correct CWD
Support for multiple backtraces?
* ~~Telescope picker support~~

