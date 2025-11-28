use std::{
    io::{Read, Write, stdin},
    net::{self, TcpListener, TcpStream},
};

fn handle_commands(mut stream: TcpStream){
    let start_sequence = [0xAB, 0xCD, 0xEF, 0x01];
    let end_sequence = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut buf = String::new();
    loop {
        print!(">");std::io::stdout().flush().unwrap();
        let std_in = stdin();
        let _ = std_in.read_line(&mut buf);
        let mut msg = [0;4096];
        msg[0..4].copy_from_slice(&start_sequence);
        msg[4] = (buf.len() as u16).to_be_bytes()[0];
        msg[5] = (buf.len() as u16).to_be_bytes()[1];
        msg[6] = 0x2;
        let length = buf.len();
        let payload_end = 7+length;
        let end_end = payload_end+4;
        msg[7..payload_end].copy_from_slice(buf.as_bytes());
        msg[payload_end..end_end].copy_from_slice(&end_sequence);

        let _ = stream.write_all(&msg[0..end_end]);
        _ = stream.flush();
        buf.clear();
        
    }
}

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
            if ![0x1, 0x3].contains(&payload[0]) {
                message.clear();
                continue;
            }
            match &payload[0] {
                0x1 => {
                    let payload = b"pong"; // 4 bytes
                    let len_bytes = (payload.len() as u16).to_be_bytes();

                    let mut response: [u8; 14] = [0; 14];

                    // start
                    response[0..4].copy_from_slice(&start_sequence);

                    // length
                    response[4] = len_bytes[0];
                    response[5] = len_bytes[1];

                    // payload
                    response[6..10].copy_from_slice(payload);

                    // end
                    response[10..14].copy_from_slice(&end_sequence);

                    stream.write_all(&response).unwrap();
                    stream.flush().unwrap();
                    message.clear();
                }

                0x3 => {
                    stream.shutdown(net::Shutdown::Both).expect("Shutdown signal sent, Stream closed");
                }

                0x2 => {
                    
                }

                _ => {
                    continue;
                }
            }
        }
    }
}
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;

    for stream in listener.incoming() {
        let stream = stream?;
        let reader = stream.try_clone()?; // for receiving
        let writer = stream.try_clone()?; // for sending

        // Thread 1: packet parser (your handle_connection)
        std::thread::spawn(move || {
            handle_connection(reader);
        });

        // Thread 2: user input -> push packet to client
        std::thread::spawn(move || {
            handle_commands(writer);
        });
    }

    Ok(())
}
