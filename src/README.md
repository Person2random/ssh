# SSH Protocol Documentation

This document defines the custom TCP-based protocol used by the SSH agent control system implemented in `main.rs`.

## Overview

The system consists of a server that listens for incoming TCP connections on `127.0.0.1:80`. Each connection is handled by two threads:
- One for receiving and parsing incoming packets from the client.
- One for reading user input from stdin and sending commands as packets to the client.

The protocol is designed for simple command and control operations, such as pinging, printing messages, and shutting down connections.

## Packet Format

All packets follow a fixed structure:

```
[Start Sequence (4 bytes)] [Length (2 bytes)] [Payload (variable)] [End Sequence (4 bytes)]
```

### Fields

- **Start Sequence**: `0xAB 0xCD 0xEF 0x01` (4 bytes)  
  Marks the beginning of a valid packet. If not matched, the packet is discarded.

- **Length**: 2 bytes (big-endian u16)  
  Specifies the length of the payload in bytes.

- **Payload**: Variable length (as specified by Length)  
  Contains the opcode and data. The first byte is the opcode, followed by data.

- **End Sequence**: `0xDE 0xAD 0xBE 0xEF` (4 bytes)  
  Marks the end of the packet. If not matched, the packet is discarded.

### Notes
- Total packet size = 10 + payload length.
- Packets are sent over TCP, so ensure proper framing as TCP is a stream protocol.
- Invalid packets (wrong start/end sequences) are silently dropped.
- The server buffers incoming data until a complete packet is received.

## Opcodes

The payload starts with a single-byte opcode, followed by opcode-specific data.

### 0x01: Ping
- **Direction**: Client to Server
- **Payload**: No additional data (length 1)
- **Response**: Server sends a packet with opcode 0x01 and payload "pong" (4 bytes)
- **Purpose**: Test connectivity.

### 0x02: Print
- **Direction**: Server to Client (from user input)
- **Payload**: String data to print (length = 1 + string length)
- **Response**: Client prints the data to stdout.
- **Purpose**: Send text commands or messages to the client.

### 0x03: Shutdown
- **Direction**: Client to Server
- **Payload**: No additional data (length 1)
- **Response**: Server closes the connection.
- **Purpose**: Gracefully terminate the session.

## Usage

### Server (main.rs)
1. Compile with `cargo build`.
2. Run with `cargo run`.
3. The server listens on `127.0.0.1:80`.
4. For each connection, it spawns threads to handle packets and user input.

### Client
- Implement a client that connects to the server and sends/receives packets according to this protocol.
- Example: Use a tool like `netcat` or write a custom client in Rust/Python.

### Example Packet Construction (Rust)
```rust
let start = [0xAB, 0xCD, 0xEF, 0x01];
let end = [0xDE, 0xAD, 0xBE, 0xEF];
let payload = [0x01]; // Ping
let length = (payload.len() as u16).to_be_bytes();
let mut packet = Vec::new();
packet.extend_from_slice(&start);
packet.extend_from_slice(&length);
packet.extend_from_slice(&payload);
packet.extend_from_slice(&end);
// Send packet over TCP stream
```

## Security Considerations
- No encryption or authentication is implemented. Use over secure networks only.
- Add TLS or shared secrets for production use.
- Validate packet lengths to prevent buffer overflows.

## Future Improvements
- Add more opcodes (e.g., for file operations).
- Implement packet acknowledgments.
- Switch to async I/O for better performance.</content>
<parameter name="filePath">d:\Coding\ssh\ssh\src\README.md