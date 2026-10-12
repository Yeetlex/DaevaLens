#!/bin/sh
# Reading the game's traffic needs CAP_NET_RAW, and nothing else: grant it to
# the binary so DaevaLens never runs as root.
setcap cap_net_raw=ep /usr/bin/daevalens || echo "DaevaLens: run 'sudo setcap cap_net_raw=ep /usr/bin/daevalens'"
exit 0
