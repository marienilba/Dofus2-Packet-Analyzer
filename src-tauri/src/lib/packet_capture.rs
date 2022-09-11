use crate::lib::packet_parse::{PacketHeader, PacketParse, ParsedPacket};
use pcap::Packet;
use std::net::IpAddr;

pub struct PacketCapture {}

impl PacketCapture {
    pub fn new() -> PacketCapture {
        PacketCapture {}
    }

    pub fn get_packet(&mut self, packet: &Packet) -> ParsedPacket {
        let data = packet.data.to_owned();
        let len = packet.header.len;
        let ts: String = format!(
            "{}.{:06}",
            &packet.header.ts.tv_sec, &packet.header.ts.tv_usec
        );

        let packet_parse = PacketParse::new();

        let parsed_packet: ParsedPacket = match packet_parse.parse_packet(data, len, ts) {
            Err(e) => {
                println!("Error reading from socket stream: {}", e);
                ParsedPacket::new()
            }
            Ok(parsed) => parsed,
        };

        return parsed_packet;
    }

    pub fn get_packet_meta(
        &self,
        parsed_packet: &ParsedPacket,
    ) -> (String, String, String, String) {
        let mut src_addr = "".to_string();
        let mut dst_addr = "".to_string();
        let mut src_port = "".to_string();
        let mut dst_port = "".to_string();

        parsed_packet.headers.iter().for_each(|pack| {
            match pack {
                PacketHeader::Tcp(packet) => {
                    src_port = packet.source_port.to_string();
                    dst_port = packet.dest_port.to_string();
                }
                PacketHeader::Udp(packet) => {
                    src_port = packet.source_port.to_string();
                    dst_port = packet.dest_port.to_string();
                }
                PacketHeader::Ipv4(packet) => {
                    src_addr = IpAddr::V4(packet.source_addr).to_string();
                    dst_addr = IpAddr::V4(packet.dest_addr).to_string();
                }
                PacketHeader::Ipv6(packet) => {
                    src_addr = IpAddr::V6(packet.source_addr).to_string();
                    dst_addr = IpAddr::V6(packet.dest_addr).to_string();
                }
                PacketHeader::Arp(packet) => {
                    src_addr = packet.src_addr.to_string();
                    dst_addr = packet.dest_addr.to_string();
                }
                _ => {}
            };
        });

        (src_addr, src_port, dst_addr, dst_port)
    }
}
