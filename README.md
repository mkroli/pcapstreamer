# pcapstreamer

pcapstreamer captures network traffic and publishes it to Kafka.

## Installation
```bash
git clone https://github.com/mkroli/pcapstreamer.git
cd pcapstreamer
cargo install
```

## Usage
```bash
pcapstreamer 0.1.0
Michael Krolikowski <mkroli@yahoo.de>


USAGE:
    pcapstreamer [OPTIONS] --kafka-servers <kafka1:9092,...> --topic <kafka topic>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter <capture filter>            berkeley packet filter
    -i, --interface <interface>              capture on specified interface
        --kafka-options <key=value,...>
    -k, --kafka-servers <kafka1:9092,...>    comma separated list of hostname:port combinations of kafka brokers
    -t, --topic <kafka topic>                kafka topic to publish data to

Make sure to exclude Kafka traffic from being captured e.g. by using a filter like "tcp port not 9092"
```
