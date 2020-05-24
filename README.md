<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/8/8c/Arrow_keys.jpg/300px-Arrow_keys.jpg" align="right" alt="c-hjkl" />

# c-hjkl

c-hjkl remaps Ctrl + HJKL into arrow keys on Linux. (X11/Wayland/Console)


## Why did you make this?

I've been using [AutoKey](https://github.com/autokey/autokey) on my Linux desktop to remap keys, but it has problems:

- It stops working on Wayland.
- It gets laggy when keys are repeatedly pressed, since it needs to run a Python script each macro keystroke.

So here I built one that solves the above problems for me. All I needed was just remapping Ctrl-H/J/K/L into arrow keys like Vim.

## How does it work?

It reads and retains exclusive access to your keyboard (`/dev/input/...`) and creates a new UInput device(a virtual keyboard) that gets piped with the keyboard events from your actual keyboard. Then, whenever it encounters a combination it wants to override (i.e. C-hjkl), it does not forward the original key events and sends the fake key events.

## Can I change the Ctrl key to something else?

No. this project's scope will be limited to C-hjkl for now. But I'll accept PRs if you make it as an option!
