extern crate rawsock;
extern crate image;
extern crate pnet;

use rawsock::open_best_library;
use std::path::Path;
use image::GenericImageView;
use image::Pixel;
use std::env;
use pnet::datalink;


const ICMP_PACKET: [u8; 62] = [
    0x00, 0x05, 0x73, 0xa0, 0x00, 0x00, 0xc8, 0x5b, 0x76, 0x3c, 0x7f, 0x2f, 0x86, 0xdd, // ethernet

    0x60, 0x0e, 0x38, 0x36, 0x00, 0x08, 0x3a, 0x40, // ipv6 header

    0x20, 0x01, 0x06, 0x7c, 0x25, 0x64, 0x03, 0x31, // source
    0x50, 0xeb, 0x9e, 0xeb, 0x71, 0xaf, 0xdb, 0x80,

    0x20, 0x01, 0x06, 0x10, 0x19, 0x08, 0xa0, 0x00, // destination
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

    0x80, 0x00, 0x25, 0x27, 0x00, 0x00, 0x00, 0x00  // X (2) Y (2) B G R A
]; 

fn main() {
    println!("Opening packet capturing library");
    let lib = open_best_library().expect("Could not open any packet capturing library");
    println!("Library opened, version is {}", lib.version());


    // Pnet
    let interf_name: String = env::args().nth(4).unwrap(); //replace with whatever is available on your platform
    let interface = datalink::interfaces().into_iter().find(|iface| iface.name == interf_name).unwrap();
    let mac = interface.mac.unwrap().octets();

    dbg!(mac);

    let mut basic_icmp_packet = ICMP_PACKET.clone();
    for i in 0..6 {
        basic_icmp_packet[i + 6] = mac[i];
    }

    println!("Opening the {} interface", interf_name);

    let interf = lib.open_interface(&interf_name).expect("Could not open network interface");
    println!("Interface opened, data link: {}", interf.data_link());

    let file = env::args().nth(1).unwrap();
    let sx = env::args().nth(2).unwrap().parse::<u32>().unwrap();
    let sy = env::args().nth(3).unwrap().parse::<u32>().unwrap();

    // Load image
    let img = image::open(&Path::new(&file)).unwrap();
    let mut packets = Vec::new();

    for (px, py, pixel) in img.pixels() {
        let c = pixel.channels();
        if c[3] > 230 {
            let mut v = basic_icmp_packet.clone();

            let x = px + sx;
            let y = py + sy;
            let l = basic_icmp_packet.len()-8;

            v[l-8] = (x >> 8) as u8;
            v[l-7] = (x & 0xff) as u8;
            v[l-6] = (y >> 8) as u8;
            v[l-5] = (y & 0xff) as u8;
            v[l-4] = c[2];
            v[l-3] = c[1];
            v[l-2] = c[0];
            v[l-1] = 255 - c[3];

            packets.push(v);
        }
    }

    //send some packets
    println!("Sending packets...");
    loop {
        for packet in &packets {
            interf.send(packet).expect("Could not send packet"); 
        }
    }

}