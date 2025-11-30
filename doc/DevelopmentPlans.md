Future Development plans
========================

After investing >400h in this project, I will have a rest at first. But I still have some ideas in which directions the project could develop:

## Support for BLE communication
The GT-120B supports BLE communication. Until now, I wasn't able to record the data transferred by the App and therefore cannot know if I can reuse anything of the existing code. But if there is at least some compatibility, I would love to support BLE too. If BLE needs a completely new communication layer implementation, it is still an option but takes much more time and energy.

## Support for other MobileAction devices
I have an old GT-120 (bought 2013) at home which still works even though the battery is not the newest anymore. The protocol is similar, but not equal. Which means: it has a completely different data format, and some other commands, but the concept of the protocol, including command format and checksum handling. I don't know how many people still use this device. There is the existing [igotu2gpx](https://launchpad.net/igotu2gpx) tool which can be used for these devices. It is quite old, but there's an [AppImage](https://github.com/daald/igotu2gpx-appimage) release, also maintained by me.
Transforming this code to my tool could be somewhat easy, but I don't think there's no real need for another tool on such outdated devices. Let me know if you see it differently.

Afaik, there's also a GT-600B device which is likely similar to the GT-120B. I don't have such a device. But If you have one, I would love to extend the support for that device. All I need from you is some testing support and a recording of the usb traffic between your device and the original software.

Any other MobileAction devices are not in my plans, but if they have little differences, I will think about it. Note that MobileAction discontinued producing these devices.

Support for other devices, not from MobileAction, are not planned. The tool focuses on the transfer protocol invented by MobileAction. For everything else, there's GPSBabel. Please check if it this tool supports your device

## GPSbabel support
In a early development phase, I wanted to implement a driver for [GPSBabel](https://www.gpsbabel.org/) instead of writing my own tool. What held me back was the programming language (C), and the fact that the tool seems to join all tracks from the device to one single file.

But now where I have a working tool, I think I or anybody could use AI one time to transform the existing code to a GPSbabel plugin. I think it's a good thought to have GPSbabel just supporting _every_ existing device on the market.

## Implement more features
At least reading the configuration would be helpful for the average user, writing the configuration too. This is probably my next task.

Updating the firmware is risky. I have one recording of a firmware update cannot repeat it. I still don't know how or from where the original software gets its firmware files or if they are bundled with the setup package. And it doesn't happen often enough that I have the feeling we need this feature. But it could be possible to implement if there's a good reason to do so, at lest for known firmware versions and not for future ones.

## Implement more options
Another thing is implementing more options: Write a single file instead of splitting by track. Write more/less information into the gpx files. Support more file formats, or a memory dump. Let me know if you need something particular and I will think about it.