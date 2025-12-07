How to test
===========

## What are these replay files?

The format of the `.pcapng.json.txt` files contain:

|prefix|purpose|
|------|--|
| `#:` |Comment lines to be printed while simulating|
| `#`  |"Invisible" comments|
| `>`  |outgoing data|
| `<`  |incoming data|

If a printed comment line contains the sequence `us=`, the program's internal time is set to this specific time. This is necessary to generate the following outgoing command (`set_time`). If it's not a simulation, the system time is used instead.

```
#: nmeaSwitch
> 93:01:01:03:00:00:00:00:00:00:00:00:00:00:00:68
< 93:00:00:6d

#: model
> 93:05:04:00:03:01:9f:00:00:00:00:00:00:00:00:c1
< 93:00:03:c2:20:15:73

[...]
```

The format of the `.pcapng.json.txt.summary` files is for information only, it contains a compact form of the same info (potentially truncated response). This file format makes it easy to compare different sessions, and for example count the number of read commands:

```
#: nmeaSwitch                            > 93:01:01:03:00:00:00:00:00:00:00:00:00:00:00:68 < 93:00:00:6d
#: model                                 > 93:05:04:00:03:01:9f:00:00:00:00:00:00:00:00:c1 < 93:00:03:c2:20:15:73
[...]
```

## Recording a session with the original software.

I used the following setup:

1. `modprobe usbmon` on linux side (plus a temporary permission fix `sudo chgrp wireshark /dev/usbmon3 && sudo chmod g+rw /dev/usbmon3`. 3 is the bus number, seen by `lsusb` (see below))
2. Installed wireshark
3. libvirt/kvm virtualization for Windows (it possibly also works with Virtualbox, but a first test had some problems)
4. Windows 11 (or whatever works with the orig sw)
5. Installed original software ([https://gpswebshop.com/products/i-gotu-gt-120b-travel-sports-gps-data-logger](https://gpswebshop.com/products/i-gotu-gt-120b-travel-sports-gps-data-logger), scroll down)

Then, do the following steps:

1. run lsusb, get the Bus number of the igotu device
2. start your windows
3. connect the USB device into the windows session (Note: you will need to do this again after a device reset. this happens after deleting the device memory, after a firmware update or after a configuration change)
4. open wireshark, start a recording of usbmon with the bus number from step 1
5. do what you like to do in the device software. for a reproducible log, I usually did a simple data download _including_ deleting the data. If you don't delete and record another session, you will see something like an delta download which only contains what's new. This mode is not supported by the linux project yet.
6. again in wireshark, stop the session, and save your session. optionally, you can already filter the data, but I prefer doing this later, see next list

Post-processing steps

1. Open your session in Wireshark, if not already open
2. Use the filter `usb.capdata[0] == 0x93 and ((usb.urb_type == 'S' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x01) or (usb.urb_type == 'C' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x81))`. It will not show everything, but it is helpful for extracting the device id(s). Scroll down, there will be multiple of them, usually in a sequence, like `3.9.1` and `3.10.1`. One after every device reconnect
3. Use another filter by using the device ids from step 2. In my case, the new filter is: `usb.addr == "3.9.1" or usb.addr == "3.9.0" or usb.addr == "3.10.1" or usb.addr == "3.10.0"`, so it's simply `.0` and `.1` to the known device ids. The `.0`'s are technically not necessary, but also belong to the device. If you want to keep a complete log of the session, you should include them.
4. Save the filtered lines by using "Export specified packats" and then filter by "All packets" and "Displayed"
5. Do a json export "Export packet dissections" / "As JSON" with the same filter
6. Now you can close wireshark
7. Run the first tool. It is used for transforming the json file to a txt file. The txt file is what you need for testing using `--sim-file-name`:

        helpers/bin/prepare-replay-txt.sh "mysession.pcapng.json" >"mysession.pcapng.json.txt"

8. A second command is optional. It helps you creating a summary file which contains one command per line, including the answer. It us useful for understanding or diffing the log:

        helpers/bin/summarize-replay-txt.sh "mysession.pcapng.json.txt" >"mysession.pcapng.json.txt.summary"



## Running igotu-gt120b against a recorded session

    cargo run -- --orig-sw-workflow --orig-sw-meta --sim-file-name sampledata/usbmon.filtered.pcapng.json.txt

The generated file should have only minor differences: The float number of the original software don't match exactly. The original software might generate 0.899999976158142 while the actual value is 0.9. Note that 0.9 is what stands in the binary data (9/10 or 90/100), so the open source implementation is the correct one here. I'm not sure where this error comes from. The original software has an optimization step which I cannot disable. It is probably the cause, or it's a problem with float arithmetics.

The two options are for convenience:

- `--orig-sw-workflow`: The original software does more calls than necessary. I assume they are used for incremental download which I didn't analyze. Replaying without this option will cause an error.
- `--orig-sw-meta` results in a base64 encoded json output of some metadata. This is not so useful, so I decided to output the same json without base64 encoidng and with some more details.

For doing a diff with the different values, I made an extra tool called `helpers/python/udiff.py`. It uses the original `difflib.py` which seems to be distributed by python itself, with a replaced compare method in `fuzzcompare.py`. The method makes `0.899999976158142` and `0.9` looking equal, even if they are part of a more complex string.