# jokolink
A crate to extract info from Guild Wars 2 MumbleLink and copy it to a file /dev/shm in linux for native linux apps (primarily jokolay).

it will also get the x11 window id of the gw2 window and paste it at the end of the mumblelink data in /dev/shm. the format is simply 1193 bytes of useful mumblelink data AND an isize (for x11 window id of gw2). will sleep for 5 ms every frame (configurable), so will copy upto 200 times per second. 


## CMD Line
Takes the path to configuration json file as argument.
example: `./jokolink.exe ./config.json` assuming both `jokolink.exe` and `config.json` are in current directory.



## Configuration
Jokolink configuration is stored in json format. 

    * loglevel: 
      default: "info"
      type: string
      possible_values: ["trace", "debug", "info", "warn", "error"]
      help: the log level of the application. 
    
    * logdir: 
        default: "." // current working directory 
        type: directory path
        help: a path to a directory, where jokolink will create jokolink.log file
    
    * mumble_link_names:
        default: ["MumbleLink"]
        type: array of strings
        help: names of mumble links to create and listen for. useful if you provide `-mumble` option to Guild Wars 2 for custom link name or if you use multi-boxing within the same prefix
    
    * interval
        default: 5
        type: unsigned integer (positive integer)
        help: the interval to sleep after updating mumble link data. in milliseconds. 5 milliseconds is roughly 200 times per second which should be enough. 
    
    * copy_dest_dir: 
        default: "z:\\dev\\shm"
        type: directory path
        help: the directory under which we will create files with the provided `mumble_link_names` and write the mumble data from the shared memory inside wine. lutris uses "z" drive to represent linux root "/". and /dev/shm is an in memory directory, so writing to files is basically just writing bytes to ram (not cached to disk filesystem).

## Script to run jokolink with lutris
sample script provided as `run_jokolink.sh`

```bash
#!/bin/bash
# WARNING!!! Jokolink will run forever unless someone else force closes it. 
# This behavior is necessary because gw2 might crash/restart itself within the same wine / lutris session. 
# so, we have no way of knowing when gw2 has quit completely within the same wine session or just restarting. 
# fortunately, lutris force closes all scripts when game exits. 


# copy this script and jokolink.exe into the wine prefix folder of gw2. and set it as the prelaunch script for gw2 in lutris. 
# lutris uses wine prefix folder as the current working directory, so keeping 
# everything relative to that will make things simpler. 

# We use a json file for configuration. if the path to config file provided doesn't exist, we just create a new config file in that place.
# Users can edit that configuration file as they see fit. documentation for the configuration can be found on the README page
# path to config file for jokolink. current directory is wine prefix, so a json file under that directory.
export config_path=./jokolink_config.json

# if jokolink crashes before initailizing logging (eg: due to bad config file), we have no way of getting errors from lutris, so 
# we output the crash stderr to this file, for easier debugging. 
# this command will use $WINE (set by lutris) and use the above options to run jokolink in background.
# remember that current directory is wine prefix, so jokolink.exe should be in that directory and the jokolink_cmd_output.txt will be created in that directory too.
$WINE './jokolink.exe' $config_path &>  jokolink_cmd_output.txt
```
## Lutris Instructions:
1. right click Guild Wars 2 in Lutris, and click on Browse Files to open the prefix folder. 
2. Copy `jokolink.exe` and `run_jokolink.sh` to that folder. 
3. right click Guild Wars 2 in Lutris again, and click on configure. make sure `Show Advanced Options` is checked at the bottom of the window.
4. go to System options tab, go down until you find the `Pre-launch script`, click on browse and select the `run_jokolink.sh` file that we pasted in prefix folder.
5. start Guild Wars 2 and you should see a `/dev/shm/{link_name}` file with link_name replaced by the mumble link name (default is "MumbleLink" ).
6. if you can't find any such file, it means jokolink probably failed to start, you can go check the prefix folder for a `jokolink.log`  file.
7. raise an issue along with that log.

### Quirks:
the Jokolink.exe will keep on running in the background until gw2 is closed. then, all the prelaunch scripts and their child process will be killed by lutris with SIGTERM signal. 

## Wine without Lutris
Jokolink needs to run with the same `runner, prefix, env` as guild-wars-2. 
1. runner. this is the wine executable that you will use to run gw2. 
2. prefix. this is the wine prefix folder of gw2, and if you use a same prefix but different runner for jokolink while gw2 is already running, it will crash.
3. environment. primarily, the variables like WINE_FSYNC. these must also match or jokolink will crash when used in the same prefix.

For best results, just use extract command like `lutris guild-wars-2 --output-script ./gw2env.sh` which will output the script to a file named `gw2env.sh`
I posted mine here for completion. username is puppy. I was able to delete most stuff except the `WINE, WINEPREFIX, WINEFSYNC` variables and launch the jokolink separately from gw2 and it worked fine. so, use those variables from your wine launching script.
replace the last line for jokolink.exe instead of guild wars 2.

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
jokolink will keep on running forever unless someone force closes it. 
so, if you are running it in background in a script, make sure to wait for gw2 to quit and kill jokolink before exiting. 


## Development
remember to make sure to run `cargo check --target=x86_64-pc-windows-gnu` to check if windows stuff is working well (if developing on linux) and `cargo check --target=x86_64-unknown-linux-gnu` to check if linux stuff is good. 

I set the default target to `x86_64-pc-windows-gnu` in the `.cargo/config.toml` along with options for linkers to mingw toolchain. in case, you use another distro and find any compile issues, that's where you need to check for issues. 
### Linux
you will need to install the mingw toolchain to crosscompile the jokolink.exe . I already set the options in `.cargo/config.toml` and usually should be good as long as you installed mingw packages. edit the run_jokolink.sh pre-launch script in lutris so that it points `./jokolink.exe` path to the `./target/x86_64-pc-windows-gnu/release/jokolink.exe` or the debug one, so that you an just do `cargo build --release` to compile it fast and launch gw2 to test if it works. or symlink them. try to log everything so that you can check the logs, instead of bothering with getting logs from lutris.
### Windows
you will need to edit the `.cargo/config.toml` and comment out everything. you will probably want to use the msvc toolchain instead of the gnu one. 
`x86_64-pc-windows-msvc` is the default target usually if you commented out the stuff as mentioned above. you will probably not run the binary part of crate as that's for wine usage, you will mostly be interested in the library part, so don't touch `main.rs`. if you want to develop for wine, use linux as its faster to check for errors live rather than dual booting back and forth to run it in wine.
