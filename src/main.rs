use std::{
    io::{Read, Write, stdin},
    net::{self, TcpListener, TcpStream},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    
};

// This program implements a simple TCP-based command and control agent.
// The server listens on port 80 and handles incoming connections.
// Each connection spawns two threads:
// 1. One to handle incoming packets from the client (handle_connection)
// 2. One to read user input and send commands to the client (handle_commands)
//
// Packet format:
// - Start sequence: [0xAB, 0xCD, 0xEF, 0x01] (4 bytes)
// - Length: 2 bytes (big-endian u16)
// - Payload: variable length
// - End sequence: [0xDE, 0xAD, 0xBE, 0xEF] (4 bytes)
//
// Opcodes:
// - 0x1: Ping request, responds with "pong"
// - 0x2: Print data to stdout
// - 0x3: Shutdown connection

fn handle_commands(mut stream: TcpStream, running: Arc<AtomicBool>) {
    // This function reads commands from stdin and sends them as packets to the connected client.
    // It constructs packets with opcode 0x2 (print command) and sends them over the stream.
    let start_sequence = [0xAB, 0xCD, 0xEF, 0x01];
    let end_sequence = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut buf = String::new();

    while running.load(Ordering::SeqCst) {
        
        print!(">");std::io::stdout().flush().unwrap();
        let std_in = stdin();
        let _ = std_in.read_line(&mut buf);
        let mut msg = [0;4096];
        if buf.trim() == "cybxzpor" {
            running.store(false, Ordering::SeqCst);
            stream.shutdown(net::Shutdown::Both).expect("Stream shutdown by you");
            break;
        }
        msg[0..4].copy_from_slice(&start_sequence);
        msg[4] = (buf.trim_end().len() as u16).to_be_bytes()[0];
        msg[5] = (buf.trim_end().len() as u16).to_be_bytes()[1];
        msg[6] = 0x2;
        let length = buf.trim().len();
        let payload_end = 7+length;
        let end_end = payload_end+4;
        msg[7..payload_end].copy_from_slice(buf.trim().as_bytes());
        msg[payload_end..end_end].copy_from_slice(&end_sequence);

        let _ = stream.write_all(&msg[0..end_end]);
        _ = stream.flush();
        buf.clear();
        
    }
}

fn handle_connection(mut stream: TcpStream, running: Arc<AtomicBool>) {
    // This function handles incoming packets from the client.
    // It parses the packet format, validates sequences, and processes opcodes.
    // For opcode 0x1 (ping), it responds with "pong".
    // For opcode 0x2 (print), it outputs the data to stdout.
    // For opcode 0x3 (shutdown), it closes the connection.
    
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
            running.store(false, Ordering::SeqCst);
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
            let opcode = payload[0];
            let data = &payload[1..];

            if ![0x1, 0x2, 0x3].contains(&payload[0]) {
                message.clear();
                continue;
            }
            match &opcode {
                0x1 => {
                    // Ping response
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
                    // Shutdown command
                    stream.shutdown(net::Shutdown::Both).expect("Shutdown signal sent, Stream closed");
                }

                0x2 => {
                    // Print command
                    std::io::stdout().write_all(&data).unwrap();
                    std::io::stdout().flush().unwrap();
                    print!(">");std::io::stdout().flush().unwrap();
                    message.clear();

                }

                _ => {
                    continue;
                }
            }
        }
    }
}
fn main() -> std::io::Result<()> {
    // Main function: Sets up a TCP listener on 127.0.0.1:80.
    // For each incoming connection, it spawns two threads:
    // - One for handling incoming packets (handle_connection)
    // - One for sending user commands (handle_commands)
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        
        let stream = stream?;
        let running = Arc::new(AtomicBool::new(true));
        let reader = stream.try_clone()?; // for receiving
        let writer = stream.try_clone()?; // for sending
        println!("New connection from {}", stream.peer_addr()?);

        let mut buf = String::new();
        let std_in = stdin();
        let _ = std_in.read_line(&mut buf);
        if buf.trim() == "0" {
            stream.shutdown(net::Shutdown::Both).expect("Stream rejected by you");
            continue;
        }

        let running_clone = running.clone();    
        // Thread 1: packet parser (your handle_connection)
        std::thread::spawn(move || {
            handle_connection(reader, running);
        });

        // Thread 2: user input -> push packet to client
        std::thread::spawn(move || {
            handle_commands(writer, running_clone);
        });
    }

    Ok(())
}
