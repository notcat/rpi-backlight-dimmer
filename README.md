# rpi-backlight-dimmer
This project changes the backlight's brightness of your Raspberry Pi 7inch touchiescreen, based off of the time of day, configured by the command line arguments.

You can express your desired brightness (0 to 1) for your local (affected by the set time on your machine) sunrise, noon, and dark using the command parameters.

The path argument is the path to your backlight file. On *all?* Raspberry Pi's, your path should be `/sys/class/backlight/rpi_backlight/brightness`

Command options are:

```less
-p, --path <path>
-s, --sunrise <sunrise>  [ default: 0.0   ]
-n, --noon <daytime>     [ default: 0.333 ]
-d, --dusk <dusk>        [ default: 0.666 ]
```

# How to compile for Raspberry Pi OS/ARM based operating systems from Windows
1. Create a folder in the project directory called `.cargo` and make a file inside of that directory called `config`

2. Inside the folder, add the following TOML configuration into the `config` file. Make sure to change `linker="<path to your linker here>"` to the path to the location of your linker. [You can find a download to the linker used to cross compile to ARMv7 here.](https://releases.linaro.org/components/toolchain/binaries/latest-5/arm-linux-gnueabihf/) Make sure to grab the `mingw32_arm-linux-gnueabihf.tar.xz` archive.

```toml
[build]
target = "armv7-unknown-linux-gnueabihf"

[target.armv7-unknown-linux-gnueabihf]
linker = "<path to your linker here>"
```

# Missing Permissions Error
You might have to run the file as root, or as a user that has permissions to the file your path is set to.