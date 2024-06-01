# Userspace driver for XP-Pen ACT05 macro keyboard with Krita keymap

This only works in Linux and was only tested on Fedora Silverblue 39.

## Layout of keys

```
( CCW=10 )   [ 0 ][ 1 ][ 2 ][ 6 ]
(   ROT  )   [ 3 ][ 4 ][ 5 ][ _ ]
( CW=11  )   [ 7 ][    8   ][ 9 ]
```

Rotary encoder sends pulses as key presses.

## ACT05 protocol

By default ACT05 acts as HID device and sends key scan codes directly. The default mapping is however too simple with too few keys that can be used by Krita.

The official application is closed source and wants to run as `root`. Not something I like.

A little bit of USB sniffing revealed that the official application sends one packet to ACT05 and switches the device to a bitmask mode, where each key press is represented by one bit in a report.

More can be seen in the [xppen_hid module](src/xppen_hid/mod.rs#L74)

## Setup

An udev rule is needed to give the input group an access to all the necessary input files

/etc/udev/rules.d/90-xppen-act05.rules

```
# XP-Pen ACT05 USB
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

### (0) Base layer

- *press* **<1>**: presses `Insert` - new paint layer
- *hold* **[3]**: holds `V` - draw a line
- *press* **<5>**: presses `B` - select the brush tool
- *press* **<6>**: presses `Ctrl-Z` - undo
- *hold* **[7]**: holds `Ctrl` and activates `layer 1`
- *hold* **[8]**: holds `Space` and activates `layer 3`
- *hold* **[9]**: holds `Shift` and activates `layer 2`
- *click* **<9>**: sends `Shift-E` - I have that mapped to `toggle eraser mode`
- **ROT**: zoom viewport

### (1) Color layer

- *click* **<0>**: presses `L` - lighter color
- *click* **<1>**: presses `Ctrl-E` - merge layer down
- *click* **<3>**: presses `K` - darker color
- *click* **<8>**: presses `Ctrl+Space` - mirror view horizontaly
- **ROT**: brush size

### (2) Shift layer

- *click* **<1>**: presses `4` - rotate viewport CCW
- *click* **<2>**: presses `6` - rotate viewport CW
- *click* **<4>**: presses `5` - reset viewport rotation
- *click* **<6>**: presses `Ctrl-Shift-Z` - redo

### (3) Space layer

- *click* **<4>**: presses `5` - reset viewport rotation
- *click* **<7>**: presses `Ctrl+Space` - mirror view horizontaly
- *hold* **[9]**: holds `Shift+Space` - rotate viewport
- **ROT**: rotate viewport


## Authors and license

Userspace driver for XP-Pen ACT05 macro keyboard with Krita keymap

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