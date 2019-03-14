/*
 * Copyright 2019 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate fern;
extern crate futures;
#[macro_use]
extern crate log;
extern crate pcap;
extern crate pcap_file;
extern crate rdkafka;
extern crate tokio;

use std::str::FromStr;

use clap::{AppSettings, Arg, ArgMatches};
use futures::Stream;

use errors::*;
use kafka_stream::kafka_stream;
use logger::setup_logger;

use crate::pcap_stream::PcapStream;

mod pcap_stream;

mod logger;

mod kafka_stream;

mod errors {
    error_chain! { }
}

fn optional_values<T: FromStr>(matches: &ArgMatches, name: &str) -> Result<Vec<T>> {
    if matches.is_present(name) {
        values_t!(matches.values_of(name), T).chain_err(|| "invalid options")
    } else {
        Ok(vec!())
    }
}

fn run() -> Result<()> {
    setup_logger()?;

    let matches: ArgMatches = app_from_crate!()
        .global_setting(AppSettings::ColoredHelp)
        .after_help("Make sure to exclude Kafka traffic from being captured e.g. by using a filter like \"tcp port not 9092\"")
        .arg(Arg::with_name("interface")
            .short("i")
            .long("interface")
            .value_name("interface")
            .help("capture on specified interface"))
        .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .value_name("capture filter")
            .help("berkeley packet filter"))
        .arg(Arg::with_name("kafka_servers")
            .short("k")
            .long("kafka-servers")
            .required(true)
            .value_name("kafka1:9092,...")
            .help("comma separated list of hostname:port combinations of kafka brokers"))
        .arg(Arg::with_name("topic")
            .short("t")
            .long("topic")
            .required(true)
            .value_name("kafka topic")
            .help("kafka topic to publish data to"))
        .arg(Arg::with_name("kafka-options")
            .long("kafka-options")
            .required(false)
            .min_values(0)
            .value_delimiter(",")
            .value_name("key=value,..."))
        .get_matches();

    let interface = matches.value_of("interface").unwrap_or("any");
    let filter = matches.value_of("filter");
    let kafka_servers = matches.value_of("kafka_servers").ok_or(Error::from("kafka missing"))?;
    let topic = matches.value_of("topic").ok_or(Error::from("topic missing"))?;
    let kafka_options = optional_values(&matches, "kafka-options").chain_err(|| "invalid kafka options")?;

    let pcap_stream = PcapStream::new(interface, filter)?;
    let stream = kafka_stream(pcap_stream, kafka_options, kafka_servers, topic.to_string())?;

    Ok(tokio::run(stream.for_each(|_| Ok(()))))
}

quick_main!(run);
