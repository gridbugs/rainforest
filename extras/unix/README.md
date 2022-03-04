# Rain Forest

## Package Contents

- rainforest-graphical: Graphical version of the game, rendering with metal on macos and vulkan on linux
- rainforest-graphical-compatibility: Graphical version of the game, using more conservative graphics libraries (e.g. preferring opengl to vulkan)
- rainforest-terminal: Terminal version of the game, rendering as text in an ansi terminal

## HIDPI

HIDPI scaling can make the game run larger than the screen size on some monitors.
The `WINIT_X11_SCALE_FACTOR` environment variable overrides the HIDPI scaling factor.

For example:
```
WINIT_X11_SCALE_FACTOR=1 ./rainforest-graphical
```
