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

