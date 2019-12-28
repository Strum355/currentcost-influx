# Current Cost CC128 to InfluxDB

[![Build Status](https://cloud.drone.io/api/badges/Strum355/currentcost-influx/status.svg)](https://cloud.drone.io/Strum355/currentcost-influx)

This program reads the output from Current Cost CC128 devices through Serial-USB and pushes data to an InfluxDB instance for graphing in Grafana.

## Usage

### CLI

```
currentcost-influx-amd64 [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --influx-url <influx-url>     [default: http://localhost:8086]
    -u, --usb-path <usb-path>         [default: /dev/ttyUSB0]
```

### SystemD

1. Copy the binary and rename to `/usr/local/bin/currentcost-influx`
2. Copy the systemd unit file to `/etc/systemd/system/`
3. Modify the systemd file as necessary (InfluxDB URL, USB device path etc). Make sure InfluxDB is up n runing!
4. Enable with `systemctl enable currentcost-influx.service` and start with `systemctl start currentcost-influx.service`
