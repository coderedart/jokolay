# jokolink
A crate to extract info from Guild Wars 2 MumbleLink and also allow to copy it to /dev/shm in linux

it will also get the x11 window id of the gw2 window and paste it at the end of the mumblelink data in /dev/shm. the format is simply 1193 bytes of useful mumblelink data AND an isize (for x11 window id of gw2). will sleep for 5 ms every frame (configurable), so will copy upto 200 times per second. 

## CMD Line Options
supports the following options in commandline arguments:
    * termlog:
        short: t
        help: max logging level to terminal stdout/stderr
        possible_values: [error, warn, info, debug, trace]
        takes_value: true
    
    * filelog:
        short: l
        help: max logging level to logfile
        possible_values: [error, warn, info, debug, trace]
        takes_value: true
    
    * logfile:
        short: f
        help: the filepath to logfile.
        takes_value: true

    * mumble:
        short: m
        help: the mumble link name that gw2 uses
        takes_value: true

    * interval:
        short: i
        help: how often jokolink copies mumble data to /dev/shm in milliseconds
        takes_value: true

    * gwcheck:
        short: g
        help: how often jokolink checks if gw2 is alive (and quits if gw2 is not alive) in seconds
        takes_value: true


## Script to run jokolink easily with lutris
sample script provided as `run_jokolink.sh`

```
#!/bin/bash

# copy this script and jokolink.exe into the prefix folder of gw2. and set it as the prelaunch script for gw2 in lutris

# variables used as options for jokolink
# these are the default values hardcoded in jokolink, but please change them if you need to. 

# the mumble link name
link_name=MumbleLink

# the log level of jokolink for terminal. valid options are "trace, debug, info, warn, error"
jl_tlog=debug

# the log lvl of jokolink for file. same options as above
jl_flog=debug

# the filename where jokolink dumps its logs to. must need write permissions to this path
jl_logfile=./jokolink.log

# the gap between each copying mumble data to /dev/shm in milliseconds. 
# 5 ms means it will copy 200 times per second roughly, as long as gw2 is less than 200 fps, we are good :)
mumble_interval=5

# the interval to check if gw2 is still running and if so quit. only useful if running outside of lutris, where we will forever be running if not for this check
# in seconds, each 5 seconds, link will check if gw2 is alive and if it is not, link will quit. usually, lutris will close us forcefully long before
gw2_check_alive=5
# 
# this command will use $WINE from lutris and use the above options to run jokolink in bg
$WINE "./jokolink.exe" -t $jl_tlog -l $jl_flog -m $link_name -f $jl_logfile -i $mumble_interval -g $gw2_check_alive & 
```
## Instructions:
1. right click Guild Wars 2 in Lutris, and click on Browse Files to open the prefix folder. 
2. Copy `jokolink.exe` and `run_jokolink.sh` to that folder. you can change the variables as you need in the script. 
3. right click Guild Wars 2 in Lutris again, and click on configure. make sure `Show Advanced Options` is checked at the bottom of the window.
4. go to System options tab, go down until you find the `Pre-launch script`, click on browse and select the `run_jokolink.sh` file that we pasted in prefix folder.
5. start Guild Wars 2 and you should see a `/dev/shm/{link_name}` file with link_name replaced by the mumble link name (usually "MumbleLink" if you didn't change the variables).
6. if you can't find any such file, it means jokolink probably failed to start, you can go check the prefix folder for a `jokolink.log` (unless you changed the variable) file.
7. raise an issue along with that log.

## Quirks:
the Jokolink.exe will keep on running in the background until gw2 is closed or crashed. then, all the prelaunch scripts and their child process will be killed by lutris with SIGTERM signal. 
## Wine without Lutris
Jokolink needs to run with the same `runner, prefix, env` as guild-wars-2. the most important ones are basically
1. runner. this is the wine executable that you will use to run gw2. 
2. prefix. this is the wine prefix folder of gw2, and if you use a same prefix but different runner for jokolink while gw2 is already running, it will crash.
3. environment. primarily, the variables like WINE_FSYNC. these must also match or jokolink crash when used in the same prefix.

For best results, just use extract command like `lutris guild-wars-2 --output-script ./gw2env.sh` which will output the script to a file named `gw2env.sh`
I posted mine here for completion. username is puppy. I was able to delete most stuff except the `WINE, WINEPREFIX, WINEFSYNC` variables and it worked fine. 
use a similar script to replace the last line for jokolink.exe and you should be good.
```
#!/bin/bash


# Environment variables
export SDL_VIDEO_FULLSCREEN_DISPLAY="off"
export VK_ICD_FILENAMES="/usr/share/vulkan/icd.d/nvidia_icd.json"
export STEAM_RUNTIME="/home/puppy/.local/share/lutris/runtime/steam"
export LD_LIBRARY_PATH="/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/lib:/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/lib64:/usr/lib/libfakeroot:/usr/lib/opencollada:/usr/lib/openmpi:/usr/lib32:/usr/lib:/usr/lib64:/home/puppy/.local/share/lutris/runtime/lib32:/home/puppy/.local/share/lutris/runtime/steam/i386/lib/i386-linux-gnu:/home/puppy/.local/share/lutris/runtime/steam/i386/lib:/home/puppy/.local/share/lutris/runtime/steam/i386/usr/lib/i386-linux-gnu:/home/puppy/.local/share/lutris/runtime/steam/i386/usr/lib:/home/puppy/.local/share/lutris/runtime/lib64:/home/puppy/.local/share/lutris/runtime/steam/amd64/lib/x86_64-linux-gnu:/home/puppy/.local/share/lutris/runtime/steam/amd64/lib:/home/puppy/.local/share/lutris/runtime/steam/amd64/usr/lib/x86_64-linux-gnu:/home/puppy/.local/share/lutris/runtime/steam/amd64/usr/lib:$LD_LIBRARY_PATH"
export DXVK_HUD="0"
export DXVK_LOG_LEVEL="none"
export STAGING_SHARED_MEMORY="1"
export __GL_SHADER_DISK_CACHE_PATH="/home/puppy/game/guild-wars-2"
export __NV_PRIME_RENDER_OFFLOAD="1"
export WINEDEBUG="-all"
export WINEARCH="win64"
export WINE="/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/bin/wine"
export GST_PLUGIN_SYSTEM_PATH_1_0="/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/lib64/gstreamer-1.0/:/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/lib/gstreamer-1.0/"
export WINEPREFIX="/home/puppy/game/guild-wars-2"
export WINEESYNC="1"
export WINEFSYNC="1"
export WINEDLLOVERRIDES="winemenubuilder.exe=d"
export WINE_LARGE_ADDRESS_AWARE="1"
export TERM="xterm"

# Command
/home/puppy/.local/share/lutris/runners/wine/lutris-gw2-6.14-3-x86_64/bin/wine '/home/puppy/game/guild-wars-2/drive_c/Program Files/Guild Wars 2/GW2-64.exe' -autologin -Windowed%       
```


### Quirk for Wine without Lutris
the following only needs to be considered IF you are not running on lutris:
> the gw2_check_alive variable in script is provided to check if gw2 process is still alive and exit if gw2 is not running.
> but this only works IF the mumblelink is initialized, so if player never reaches it will keep on running forever. this is a feature, as we don't know how long the user will keep on being afk at the character select screen. i myself often start gw2 and go make snacks etc.. and i don't want to restart gw2 when i am back because jokolink killed itself waiting for me to login. but i will expose a timeout feature IF someone asks for it :) with a default of no timeout.
> I suggest removing the trailing `&`, then the script will block until you close it yourself, which will kill jokolink too.

## Development
As this crate contains both linux specific and windows specific stuff, I recommend keeping the linux stuff in linux.rs and windows stuff in win.rs . 
Also, remember to make sure to run `cargo check --target=x86_64-pc-windows-gnu` to check if windows stuff is working well and `cargo check --target=x86_64-unknown-linux-gnu` to check if linux stuff is good. 
goes without saying, but you need `rustup.rs` to install both of the above target's support on linux. 
I set the default target to `x86_64-pc-windows-gnu` in the `.cargo/config.toml` along with options for linkers to mingw toolchain. in case, you use another distro and find any compile issues, that's where you need to check for issues. 
### Linux
you will need to install the mingw toolchain to crosscompile the jokolink.exe . I already set the options in `.cargo/config.toml` and usually should be good as long as you installed mingw packages. edit the run_jokolink.sh pre-launch script in lutris so that it points to the `./target/x86_64-pc-windows-gnu/release/jokolink.exe` or the debug one, so that you an just do `cargo build --release` to compile it fast and launch gw2 to test if it works. try to log everything so that you can check the logs, instead of bothering with getting logs from lutris.
### Windows
you will need to edit the `.cargo/config.toml` and comment out everything. you will probably want to use the msvc toolchain instead of the gnu one. 
`x86_64-pc-windows-msvc` is the default target usually if you commented out the stuff as mentioned above. you will probably not run the binary part of crate as that's for wine usage, you will mostly be interested in the library part, so don't touch `main.rs`. if you want to develop for wine, use linux as its faster to check for errors live rather than dual booting back and forth to REPL.

## Contributing
please make an issue BEFORE you start work on anything. Most of the work remaining is just getting it more type safe with wrapper types like for example
```
#[repr(transparent)]
pub struct MapID(u32)
```
We will need such wrapper types for fields of MumbleLink, also Units based wrapper types for continent coords or pixels or radians etc..
All of this is blocked on my JokoApi crate where i plan to introduce all the API exposed types. 