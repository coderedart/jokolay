#!/bin/bash

# copy this script and jokolink.exe into the prefix folder of gw2. and set it as the prelaunch script for gw2 in lutris

# variables used as options for jokolink
# these are the default values hardcoded in jokolink, but please change them if you need to. 

# the mumble link name
link_name=MumbleLink

# the log lvl of jokolink for file. same options as above
jl_log_level=debug

# the filename where jokolink dumps its logs to. must need write permissions to this path
jl_logfile_dir=.

# the gap between each copying mumble data to /dev/shm in milliseconds. 
# 5 ms means it will copy 200 times per second roughly, as long as gw2 is less than 200 fps, we are good :)
mumble_interval=5

# the interval to check if gw2 is still running and if so quit. only useful if running outside of lutris, where we will forever be running if not for this check
# in seconds, each 1 seconds, link will check if gw2 is alive and if it is not, link will quit. if lutris doesn't force close the link, we can use this as a backup to quit 
# when gw2 closes. this also serves as the interval to check for mumble initialize or gw2's xid at the start. this is very cheap, so its fine to set it to a second.

gw2_check_alive=1

# usually, drive Z is sym linked in lutris/wine to our linux root "/" . we use /dev/shm, but specify the path in "windows" style of backward slashes while escaping them
# you can change MumbleLink to something else like "Alt1" or "Alt2" etc.. for multi boxing, while keeping the link_name same for all of them. we can later modify the script
# to have multiple choices of this variable and choose one based on the arguments to the script itself. we can use the same script for all the gw2 instances with just the argument being different
dest_file_path='z:\\dev\\shm\\MumbleLink'
# 
# this command will use $WINE from lutris and use the above options to run jokolink in bg
$WINE "./jokolink.exe" -l $js_log_level -m $link_name -d $jl_logfile_dir -i $mumble_interval -g $gw2_check_alive -f $dest_file_path & 

