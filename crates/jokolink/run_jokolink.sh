#!/bin/bash
# WARNING!!! Jokolink will run forever unless someone else force closes it. 
# This behavior is necessary because gw2 might crash/restart itself.
# so, we have no way of knowing when gw2 has quit completely within the same wine session. 
# fortunately, lutris force closes all scripts. 
# we could make it a daemon so it will keep running forever, but then you cannot change 
# runners or their settings in lutris while jokolink is running in background. 



# copy this script and jokolink.exe into the wine prefix folder of gw2. and set it as the prelaunch script for gw2 in lutris

# We use a json file for configuration. if the path to config file provided doesn't exist, we just create a new config file in that place.
# Users can edit that configuration file as they see fit. documentation for the configuration can be found on the README page
# path to config file for jokolink 
export config_path=./jokolink_config.json

# if jokolink crashes before initailizing logging, we have no way of getting errors, so 
# we output the crash stderr to this file, for easier debugging. 
# this command will use $WINE from lutris and use the above options to run jokolink in bg
$WINE './jokolink.exe' --config $config_path &>  jokolink_cmd_output.txt