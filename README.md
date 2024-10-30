# Userspace driver for XP-Pen ACK05 macro keyboard with Krita keymap

This only works in Linux and was only tested on Fedora Silverblue 39.

## Layout of keys

```
( CCW=10 )   [ 0 ][ 1 ][ 2 ][ 6 ]
(   ROT  )   [ 3 ][ 4 ][ 5 ][ _ ]
( CW=11  )   [ 7 ][    8   ][ 9 ]
```

or when rotated

```
 [ 9 ][    8   ][ 7 ]
 [ 6 |[ 5 ][ 4 ][ 3 ]
 | _ ][ 2 ][ 1 ][ 0 ]  ( CCW=10 ROT CW=11 )
```

Rotary encoder sends pulses as key presses.

## ACK05 protocol

By default ACK05 acts as HID device and sends key scan codes directly. The default mapping is however too simple with too few keys that can be used by Krita.

The official application is closed source and wants to run as `root`. Not something I like.

A little bit of USB sniffing revealed that the official application sends one packet to ACK05 and switches the device to a bitmask mode, where each key press is represented by one bit in a report.

More can be seen in the [xppen_hid module](src/xppen_hid/mod.rs#L74)

## Setup

An udev rule is needed to give the input group an access to all the necessary input files

/etc/udev/rules.d/90-xppen-ack05.rules

```
# XP-Pen ACK05 USB
ATTR{idVendor}=="28bd", ATTR{idProduct}=="0202", MODE="660", GROUP="input"

# /dev/uinput to allow virtual keyboards
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
```

Then add your user to the input group.

At last, reload udev daemon and reconnect the keypad.

```
sudo udevadm control --reload
```

## Build

- Make sure you have the development libraries for udev and hid installed. Those differ between systems. My Fedora uses `systemd-devel` and `systemd-udev`.
- Install Rust and cargo, preferably using `rustup` (https://www.rust-lang.org/tools/install)
- Build using `cargo build`
- Start using `cargo run`

## Keymap

The included keymap is designed to help with painting in Krita.

Once the application is running a multilayer keymap should be active and behave like this.

The keymap can be modified in the [load_layout](src/layout/serialization.rs#L18) function (for now, I plan to eventually separate it from the code.)

```
( CCW <- )   [ 0 ][ 1 ][ 2 ][ 6 ]
(   ROT  )   [ 3 ][ 4 ][ 5 ][ _ ]
( -> CW  )   [ 7 ][    8   ][ 9 ]
```

```
 [ 9 ][    8   ][ 7 ]  ( CCW <- 10 )
 [ 6 |[ 5 ][ 4 ][ 3 ]      ROT
 | _ ][ 2 ][ 1 ][ 0 ]  ( ->  CW 11 )
```

### (0) Base layer

- *long* **<2>**: presses `Delete` - clear layer
- *hold* **[3]**: activates `layer 3 - view and move`
- *click* **<4>**: presses `B` - select the brush tool
- *hold* **[4]**: activates `layer 1 - colors`
- *hold* **[5]**: activates `layer 4 - drawing` (which holds `V` - line mode)
- *press* **<6>**: presses `Ctrl-Z` - undo
- *click* **<7>**: presses `Insert` - new paint layer
- *hold* **[7]**: activates `layer 5 - layers`
- *click* **<8>**: sends `Shift-E` - I have that mapped to `toggle eraser mode`
- *hold* **[8]**: activates `layer 2 - tools`
- *click* **[9]**: Sends `F12` - I have that mapped to `freehand selection tool`
- *long* **[9]**: Sends `Ctrl+Shift+A` - clear selection
- **ROT**: zoom viewport

### (1) Color and painting layer

When this layer is active you can tap with the stylus to
pick color (it holds `Ctrl`).

- *click* **<3>**: presses `K` - darker color
- *click* **<7>**: presses `L` - lighter color
- *click* **<8>**: presses `Ctrl+Space` - mirror view horizontaly
- **ROT**: brush size

### (2) Tool layer

When this layer is active you can drag with the stylus to change
brush size.

- *click* **<0>**: presses `ESC` - cancel
- *click* **<1>**: presses `5` - reset viewport rotation
- *click* **<2>**: presses `Ctrl-T` - transform tool
- *click* **<4>**: presses `Enter` - confirm
- *click* **<7>**: presses `Ctrl+Space` - mirror view horizontaly
- *click* **<9>**: presses `T` - move layer

### (3) View and move layer

When this layer is active you can drag with the stylus to move
canvas view (it holds `space`).

- *click* **<4>**: presses `5` - reset viewport rotation
- *click* **<6>**: presses `Ctrl-Shift-Z` - redo
- *click* **<8>**: presses `Ctrl-Space` - mirror viewport
- **ROT**: rotate viewport

### (4) Drawing layer

When this layer is active it holds `V` which allows drawing
straight lines.

### (5) Layers layer

- *click* **<8>**: presses `Ctrl-E` - merge layer down

## Authors and license

Userspace driver for XP-Pen ACK05 macro keyboard with Krita keymap

Copyright (C) 2024  Martin Sivak

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a [copy of the GNU General Public License](LICENSE)
along with this program.  If not, see <https://www.gnu.org/licenses/>.
