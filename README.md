i-gotU GT-120B GPS Logger Driver
================================

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A user-space software for the MobileAction i-gotU GT-120B GPS Data Logger

Developed using [Rust](https://rust-lang.org/), Tested for Linux but might also work under Windows and MacOS.

Links to hardware info:

* Comparison table: [local copy](doc/devicespec/GPS_logger_comparison.pdf), [remote](https://www.mobileaction.com/igotu-gps/GPS_logger_comparison.pdf)
* Shop page with detailed specs: [local copy](doc/devicespec/i-gotU_GT-120B_GPSWebShop.pdf), [remote](https://gpswebshop.com/products/i-gotu-gt-120b-travel-sports-gps-data-logger)

## Downloading GPS Tracks

Download without deleting data from device (download-only, repeatable):

    igotu-gt120

Download and wipe device

    igotu-gt120 --clear

## More documentation
* [Further development plans](doc/DevelopmentPlans.md)

## Tool safety
Status: Working. I didn't heavily test with devices, but I tested a lot using recorded sessions of the original software, so I can say I'm quite sure that this device will not behave differently than the original software.

The tool only deletes data from the device if everything was downloaded and saved successfully to disk AND the option `--clear` is activated. In any error case, the tool stops before starting the delete procedure.
Anyways, there's always a little risk on free software, actually _every_ software, that something goes wrong.

If in doubt, do a first run without the `--clear` option and check the output.

## Credits
Many thanks to [igotu2gpx](https://launchpad.net/igotu2gpx) which is the open source implementation for the older version of this device, GT-120 (without B). It helped me understanding the protocol, but I copied only very few codelines (e.g. checksum calculation) from there.

## License
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see https://www.gnu.org/licenses/.
