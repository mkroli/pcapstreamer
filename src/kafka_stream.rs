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

use std::str::FromStr;
use std::time::Duration;

use futures::Stream;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tokio::prelude::*;

use crate::errors::*;

#[derive(Debug)]
pub struct KafkaOption {
    key: String,
    value: String,
}

impl FromStr for KafkaOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let pivot = s.find('=').ok_or(Error::from("invalid format for kafka option"))?;
        let (key, value) = s.split_at(pivot);
        let value = &value[1..];
        Ok(KafkaOption {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

pub fn kafka_stream<I>(input: I, kafka_options: Vec<KafkaOption>, kafka_server: &str, topic: String) -> Result<impl Stream<Item=(), Error=()>> where I: Stream<Item=Vec<u8>, Error=Error> + Sized {
    let mut kafka_config = ClientConfig::new();
    kafka_options.iter().fold(&mut kafka_config, move |config, option| {
        config.set(&option.key, &option.value)
    });
    let kafka: FutureProducer = kafka_config
        .set("client.id", "pcapstreamer")
        .set("bootstrap.servers", kafka_server)
        .create()
        .chain_err(|| "failed to create kafka producer")?;

    let kafka_stream = input
        .and_then(move |packet| {
            let record = FutureRecord {
                topic: &topic,
                partition: None,
                payload: Some(&packet),
                key: None as Option<&str>,
                timestamp: None,
                headers: None,
            };
            kafka.send(record, 0).map_err(|e| Error::with_chain(e, "failed to publish packet"))
        })
        .and_then(|x| match x {
            Ok(_) => Ok(()),
            Err((e, _)) => {
                warn!("publishing to kafka failed: {}", e);
                Err(Error::with_chain(e, "publishing to kafka failed"))
            }
        })
        .timeout(Duration::from_millis(1000))
        .then(|_| Ok(()));

    Ok(kafka_stream)
}
