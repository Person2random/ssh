use std::{
    io::{Read, Write},
    net::{self, TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    let mut length: u16 = 0;
    let mut message = Vec::new();
    let start_len = 4;
    let end_len = 4;
    let start_sequence = [0xAB, 0xCD, 0xEF, 0x01];
    let end_sequence = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut buf = [0; 4096];
    loop {
        let read = stream.read(&mut buf).unwrap();
        if read == 0 {
            stream
                .shutdown(net::Shutdown::Both)
                .expect("Stream abruptly shutdown by server");
            break;
        }
        message.extend_from_slice(&buf[0..read]);
        if message.len() >= 6 {
            if message[0..start_len] != start_sequence {
                message.clear();
                continue;
            }
            length = u16::from_be_bytes([message[4], message[5]])
        }

        if message.len() >= (6 + length + end_len).into() {
            let end_start = 6 + length as usize;
            let end_of_packet = end_start + end_len as usize;
            if &message[end_start..(end_of_packet as usize)] != end_sequence {
                message.clear(); //Drop the message (If there is a valid start after this end it is also dropped, Keep this in mind when developing the client)
                continue;
            }
            stream.flush().unwrap();
            let payload_end = 6 + length as usize;
            let payload = &message[6..payload_end];
            if ![0x1,0x2,0x3,0x4].contains(&payload[0]){
                message.clear();
                continue;
            }
            match &payload[0] {
                0x1=>{
                    let mut response: [u8; 13] = [0;13];
                    response[0..4].copy_from_slice(&start_sequence);
                    response[4] = 0x1;
                    response[5..9].copy_from_slice("pong".as_bytes());
                    response[9..13].copy_from_slice(&end_sequence);
                    _ = stream.write_all(&response);
                    stream.flush().unwrap();
                    message.clear();
                }


                _=>{
                    continue;
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("Connection failed {e}")
            }
        }
    }

    return Ok(());
}
