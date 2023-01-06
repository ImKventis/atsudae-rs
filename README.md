
# Atsudae
A rewrite of a daemon for my broken system [originally in C](https://github.com/ImKventis/powerthing)

## The problem
My laptop has a very odd CPU boosting problem where it will boost to the high heavens no matter what the temperature is
until it completely powers off causing me to lose all my work which isn't ideal. 

I have taken the laptop apart, cleaned and replaced the thermal paste/pads but this has little to no effect, so I had an
idea to create a daemon to manually turn off the boost using the sysfs located on /sys/

## How it works
The intel_pstate driver has a file at `/sys/.../cpu/intel_pstate/no_turbo` which can be used to as you would
expect to disable CPU boosting, of course I can just use this to disable the boost entirely, but I still want to have CPU
boosting so this daemon also uses lm_sensors to figure out the CPU temperature and act on that. Its quite simple overall.

When lm_sensors are unavailable the application wil simply watch `no_turbo` to see if it changes back to a `0`, for some reason setting `no_turbo`
to `1` on boot works but only temporary...

## Stdin Commands
While the program is running in a terminal it will accept a few basic commands to change settings on the fly.

### Available commands

- boost - Starts using the boost loop
- noboost - Stops using the boost loop and starts using the non-boost loop
- clear - Clears the screen
- help - Shows a help message

## Configuration
I don't believe any would need to use this program, but I have created some basic configuration for the program

```conf
# These are commets
BOOST=true;
LOGLEVEL=2;
LOGFILE=/var/log/atsudae.log;
MAXFAIL=5;
STDIO=false;
```
Alternative version
```conf
BOOST=true;LOGLEVEL=2;LOGFILE=/var/log/atsudae.log;MAXFAIL=5;
```
This is the basic and default configuration. You do not need to separate with newlines but the semicolon is needed.

- BOOST - Whether the program should boost the CPU when it's cold.
- LOGLEVEL - The loglevel for the program to use, see log levels below
- LOGFILE - The logger will output to stdout but also a file, this will be the file to output to
- MAXFAIL - How many times the program is allowed to fail the basic loop, if this is reached then the program will resort
to non boost mode. 
- STDIO - Listen to commands via stdin

### log levels

- 1 - Errors Messages Only
- 2 - Errors and Warning Messages Only
- 3 - Errors,Warning and Info messages Only
- 4 - All messages, Errors, Warning, Info, Debug
