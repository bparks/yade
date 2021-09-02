use std::convert::TryInto;
use hex::FromHex;
use std::io::prelude::*;
use std::net::TcpListener;
use std::fs;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Listening!");
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        println!("Connection established!");

        // Let's welcome the client to our world
        let welcome = Vec::from_hex("590000000a352e352e352d31302e342e31312d4d61726961444200090000005b462f554126367d00fef7210200ff8115000000000000070000003631334d6a4f2d2f454d355d006d7973716c5f6e61746976655f70617373776f726400").unwrap();
        stream.write(&welcome).unwrap();

        // They should respond with an authentication attempt
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        // To which we'll respond with "OK"
        let ok_packet = Vec::from_hex("0700000200000002000000").unwrap();
        stream.write(&ok_packet).unwrap();

        // From here on, client will send us commands; we should handle them reasonably
        loop {
            for elem in buffer.iter_mut() { *elem = 0; }
            stream.read(&mut buffer).unwrap();
            //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

            // Layout of message is 3 bytes for length, 1 byte for packet #, 1 byte for command
            let packet_num = buffer[3];
            let command = buffer[4];
            if command == 0x0e { // Ping
                let ping_ok = Vec::from_hex("0700000100000002000000").unwrap();
                stream.write(&ping_ok).unwrap();
            } else if command == 0x03 { // Query
                //let packet_length = buffer[2] << 16 + buffer[1] << 8 + buffer[0] - 1;
                let statement = &buffer[5..];
                println!("Query: {}", String::from_utf8_lossy(statement));

                if String::from_utf8_lossy(statement).starts_with("select * from things") {
                    let list_of_files = fs::read_dir("./data").unwrap().filter_map(|f| f.ok().and_then(|e| e.path().file_name().and_then(|n| n.to_str().map(|s| String::from(s))))).collect();
                    let ping_ok = build_response(list_of_files);
                    //println!("{:02x?}", &ping_ok);
                    stream.write(&ping_ok).unwrap();
                } else {
                    let ping_ok = Vec::from_hex("01000001021b00000203646566000000055461626c65000c2d0000010000fd010027000022000003036465660000000c437265617465205461626c65000c2d0000100000fd010027000005000004fe00000200f90100050a77705f6f7074696f6e73fceb01435245415445205441424c45206077705f6f7074696f6e736020280a2020606f7074696f6e5f69646020626967696e742832302920756e7369676e6564204e4f54204e554c4c204155544f5f494e4352454d454e542c0a2020606f7074696f6e5f6e616d65602076617263686172283139312920434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2044454641554c542027272c0a2020606f7074696f6e5f76616c756560206c6f6e677465787420434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2c0a2020606175746f6c6f61646020766172636861722832302920434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2044454641554c542027796573272c0a20205052494d415259204b45592028606f7074696f6e5f696460292c0a2020554e49515545204b455920606f7074696f6e5f6e616d65602028606f7074696f6e5f6e616d6560292c0a20204b455920606175746f6c6f6164602028606175746f6c6f616460290a2920454e47494e453d496e6e6f4442204155544f5f494e4352454d454e543d3134302044454641554c5420434841525345543d757466386d623420434f4c4c4154453d757466386d62345f756e69636f64655f636905000006fe00000200").unwrap();
                    stream.write(&ping_ok).unwrap();
                }
            } else {
                println!("Unhandled message {}; packet number {}", command, packet_num);
                let ping_ok = Vec::from_hex("0700000100000002000000").unwrap();
                stream.write(&ping_ok).unwrap();
            }
        }
    }
}

// Example query response
// 010000 01 02
// 1b0000 02 03646566000000055461626c65000c2d 0000010000fd0100270000
// 220000 03 036465660000000c4372656174652054 61626c65000c2d0000100000fd010027 0000
// 050000 04 fe00000200
// f90100 05 0a77705f6f7074696f6e73fceb01435245415445205441424c45206077705f6f7074696f6e736020280a2020606f7074696f6e5f69646020626967696e742832302920756e7369676e6564204e4f54204e554c4c204155544f5f494e4352454d454e542c0a2020606f7074696f6e5f6e616d65602076617263686172283139312920434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2044454641554c542027272c0a2020606f7074696f6e5f76616c756560206c6f6e677465787420434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2c0a2020606175746f6c6f61646020766172636861722832302920434f4c4c41544520757466386d62345f756e69636f64655f6369204e4f54204e554c4c2044454641554c542027796573272c0a20205052494d415259204b45592028606f7074696f6e5f696460292c0a2020554e49515545204b455920606f7074696f6e5f6e616d65602028606f7074696f6e5f6e616d6560292c0a20204b455920606175746f6c6f6164602028606175746f6c6f616460290a2920454e47494e453d496e6e6f4442204155544f5f494e4352454d454e543d3134302044454641554c5420434841525345543d757466386d623420434f4c4c4154453d757466386d62345f756e69636f64655f6369
// 050000 06 fe00000200

fn build_response(results: Vec<String>) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();

    let mut pkt_no = 1;

    pkt_no += build_field_headers(&mut buf, pkt_no);
    pkt_no += build_eof(&mut buf, pkt_no, 0x0022);
    for item in results {
        pkt_no += build_row(&mut buf, pkt_no, item);
    }
    build_eof(&mut buf, pkt_no, 0x0002);

    return buf;
}

fn build_field_headers(buf: &mut Vec<u8>, mut pkt_no: u8) -> u8 {
    // How many fields?
    buf.push(1); // Length
    buf.push(0);
    buf.push(0);
    buf.push(pkt_no); // Packet number
    buf.push(1); // Number of fields
    pkt_no = pkt_no + 1;

    // First field
    let packet_len = "def".len() + "demo".len() + "demotable".len() + "demotable".len() + "col".len() + "col".len() + 6 + 1 + 2 + 4 + 1 + 2 + 1 + 2;

    buf.push((packet_len).try_into().unwrap());
    buf.push(0);
    buf.push(0);
    buf.push(pkt_no);
    len_coded_string(buf, "def"); // Catalog
    len_coded_string(buf, "demo"); // Database
    len_coded_string(buf, "demotable"); // Table
    len_coded_string(buf, "demotable"); // Original Table
    len_coded_string(buf, "col"); // Column name
    len_coded_string(buf, "col"); // Orginal name
    buf.push(0x0c); // Length of fixed length fields
    buf.push(0x2d); // Charset (utf8mb4)
    buf.push(0);
    buf.push(80); // Length
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(253); // Type (string)
    buf.push(0); // Flags
    buf.push(0);
    buf.push(0); // Decimals
    buf.push(0); // Filler
    buf.push(0);
    //pkt_no = pkt_no + 1;

    return 2;
}

fn len_coded_string(buf: &mut Vec<u8>, string: &str) {
    buf.push(string.len().try_into().unwrap());
    for byte in string.as_bytes() {
        buf.push(*byte);
    }
}

fn build_eof(buf: &mut Vec<u8>, pkt_no: u8, status: u16) -> u8 {
    buf.push(5); // Length
    buf.push(0);
    buf.push(0);
    buf.push(pkt_no); // Packet number
    buf.push(254); // EOF
    buf.push(0); // Warnings
    buf.push(0);
    buf.push((status & 0xff).try_into().unwrap()); // Server status
    buf.push((status >> 8).try_into().unwrap());

    return 1;
}

fn build_row(buf: &mut Vec<u8>, pkt_no: u8, item: String) -> u8 {
    buf.push((item.len() + 1).try_into().unwrap()); // Length
    buf.push(0);
    buf.push(0);
    buf.push(pkt_no);
    len_coded_string(buf, item.as_str());
    
    return 1;
}