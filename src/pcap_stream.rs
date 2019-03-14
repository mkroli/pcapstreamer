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

use futures::Stream;
use pcap::{Active, Capture, Device, Linktype};
use pcap_file::{DataLink, PcapHeader, PcapWriter};
use tokio::prelude::*;

use crate::errors::*;

pub struct PcapStream {
    capture: Capture<Active>,
    header: PcapHeader,
}

impl PcapStream {
    pub fn new(interface: &str, filter: Option<&str>) -> Result<Self> {
        let devices = Device::list()
            .chain_err(|| "error listing interfaces")?;
        let device = devices
            .into_iter()
            .find(|d| d.name == interface)
            .ok_or(Error::from("interface not found"))?;
        let mut capture = device
            .open()
            .chain_err(|| "failed to open interface")?
            .setnonblock()
            .chain_err(|| "failed to set nonblock")?;
        if let Some(filter) = filter {
            capture.filter(filter).chain_err(|| "invalid filter")?;
        }
        let Linktype(link_type) = capture.get_datalink();
        let header = PcapHeader::with_datalink(DataLink::from(link_type as u32));

        Ok(PcapStream {
            capture: capture,
            header: header,
        })
    }
}

impl Stream for PcapStream {
    type Item = Vec<u8>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.capture.next() {
            Err(pcap::Error::TimeoutExpired) => {
                Ok(Async::NotReady)
            }
            Err(e) => {
                Err(Error::with_chain(e, "failed to capture packet"))
            }
            Ok(packet) => {
                let buffer = Vec::with_capacity(65 * 1024);
                let mut pcap_writer: PcapWriter<Vec<u8>> = PcapWriter::with_header(self.header, buffer).chain_err(|| "failed to write packet")?;
                let p = pcap_file::Packet::new(
                    packet.header.ts.tv_sec as u32,
                    packet.header.ts.tv_usec as u32,
                    packet.header.len,
                    packet.data,
                );
                pcap_writer.write_packet(&p).chain_err(|| "failed to write packet")?;
                let buffer = pcap_writer.into_writer();
                Ok(Async::Ready(Some(buffer)))
            }
        }
    }
}
