# jokolink
A crate to extract info from Guild Wars 2 MumbleLink and copy it to a file /dev/shm in linux for native linux apps (primarily jokolay).

it will also get the x11 window id of the gw2 window and paste it at the end of the mumblelink data in /dev/shm. the format is simply 1193 bytes of useful mumblelink data AND an isize (for x11 window id of gw2). will sleep for 5 ms every frame (configurable), so will copy upto 200 times per second. 

## Precaution
This jokolink binary is ONLY for linux users to get the `MumbleLink` data from guild wars 2 in wine to `/dev/shm`, so that linux native clients can read that. eg: `Jokolay`. 

> WARNING: Guild Wars 2 doesn't update MumbleLink Data during character select screen or map loading screens. So, until you load into a map with a character, there is nothing for jokolink to write to `/dev/shm/MumbleLink`

## Installation
1. Just run `cargo build -p jokolink --release` to build the `jokolink.dll` (or download it )
2. copy the `jokolink.dll` into `Guild Wars 2` folder right beside `Gw2-64.exe`
3. If you don't use arcdps, then rename `jokolink.dll` to `d3d11.dll`, so that gw2 will load the dll when it starts
4. If you use arcdps, then you can rename `jokolink.dll` to `arcdps_jokolink.dll`. All dlls whose names start with `arcdps` will be loaded by arcdps. 


## Configuration
Jokolink configuration is stored in json format and a default config file will be created in the same directory as the dll. 

    * loglevel: 
      default: "info"
      type: string
      possible_values: ["trace", "debug", "info", "warn", "error"]
      help: the log level of the application. 
    
    * logdir: 
        default: "." // current working directory 
        type: directory path
        help: a path to a directory, where jokolink will create jokolink.log file
    
    * mumble_link_name:
        default: "MumbleLink"
        type: string
        help: names of mumble link to copy data from and to. useful if you provide `-mumble` option to Guild Wars 2 for custom link name
    
    * interval
        default: 5
        type: unsigned integer (positive integer)
        help: the interval to sleep after updating mumble link data. in milliseconds. 5 milliseconds is roughly 200 times per second which should be enough. 
    
    * copy_dest_dir: 
        default: "z:\\dev\\shm"
        type: directory path
        help: the directory under which we will create files with the provided `mumble_link_names` and write the mumble data from the shared memory inside wine. lutris uses "z" drive to represent linux root "/". and /dev/shm is an in memory directory, so writing to files is basically just writing bytes to ram (not wrriten to ssd/hdd -> really fast copying).


## Verification :
1. start Guild Wars 2 and you should see a file at `/dev/shm/MumbleLink`. If you use a custom link name by editing the config, then the path will be `/dev/shm/custom_link_name`. 
2. The jokolink dll is basically copying gw2 data to this file. you can either do `cat /dev/shm/MumbleLink` or use a hex editor to browse the data. If you are playing in a PvE map, then you should see the currently logged in player name easily.
3. if you can't find any such file, it means jokolink probably failed to start, you can go check the `Guild Wars 2` folder for `jokolink.log` and raise an issue with that log.
4. If you right click the game in lutris and select `show logs`, you can see lines printed by jokolink when it is loaded/unloaded and initialized. 



## Cross Compilation
To compile for windows on linux, install `x86_64-pc-windows-gnu` target with rustup and `mingw` package on your distro. 
`.cargo/config.toml` already sets the linker settings for mingw toolchain.
