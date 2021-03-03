use nix::unistd::{getuid, Uid};
use smol::io::BufReader;
use smol::prelude::*;
use smol::{io::BufWriter, net::unix::UnixStream};
use std::convert::TryFrom;

use crate::type_system::*;
use crate::type_system::types::*;
use crate::message_protocol::MessageType;
use crate::message_protocol::body::Body;
use crate::message_protocol::header::{HeaderField};


#[derive(Debug)]
pub struct Connection {
    serial: u32,
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

fn get_uid_as_hex() -> String {
    // Get UID
    let uid: Uid = getuid();
    let uid: u32 = uid.as_raw();
    // Convert it to a string, "1000" for example.
    let uid: String = format!("{}", uid);
    // Encode the "1000" string as lowercase hex, fx "31303030", which is the format
    // that the DBus auth protocol wants.
    let uid: String = hex::encode(uid);

    uid
}

impl Connection {
    pub async fn new_system() -> crate::Result<Self> {
        // TODO check DBUS_SYSTEM_BUS_ADDRESS env variable, if it is set, connect to that instead.
        log::info!("Connecting to dbus socket.");
        let stream = UnixStream::connect("/var/run/dbus/system_bus_socket").await?;

        // Split up into buffered read/write.
        let reader = BufReader::new(stream.clone());
        let writer = BufWriter::new(stream);

        let mut conn = Connection {
            reader,
            writer,
            serial: 0,
        };

        // Spec for some reason requires that the first thing we do is to send a null byte.
        log::debug!("Writing null byte.");
        conn.writer.write(&[0]).await?;

        log::info!("Authenticating.");
        conn.auth().await?;

        log::debug!("Saying hello.");
        conn.say_hello().await?;

        Ok(conn)
    }

    pub async fn call_method(
        &mut self,
        destination: DBusString,
        path: DBusObjectPath,
        interface: DBusString,
        member: DBusString,
        body: Body,
    ) -> crate::Result<()> {
        todo!("This is undone, several things probably missing.");

        let endianness: Endianness = Endianness::BigEndian;
        let message_type: MessageType = MessageType::MethodCall;
        let flag_vec: Vec<HeaderFlag> = vec![]; // fix?

        let header_fields: Vec<HeaderField> = vec![
            HeaderField::Destination(destination),
            HeaderField::Interface(interface),
            HeaderField::Member(member),
            HeaderField::Path(path),
        ];

        // TODO
        let body_serialized: Vec<u8> = vec![];

        self.serial += 1;
        let serial_of_this_message = self.serial;

        let header_fields: HeaderFields = {
            let mut array: HeaderFields = DBusArray { vec: vec![] };

            for header_field in header_fields {
                let decimal_code: u8 = header_field.decimal_code();
                let variant: DBusVariant = DBusVariant::from(header_field);

                let dbus_struct = (DBusByte(decimal_code), variant);
                array.vec.push(dbus_struct);
            }

            array
        };

        let mut header_serialized: Vec<u8> = {
            let mut v: Vec<u8> = Vec::new();

            v.push(endianness.serialize());
            v.push(message_type.decimal_value());

            let mut flags = 0;
            for flag in flag_vec {
                flags |= flag.hex_value();
            }
            v.push(flags);

            v.push(crate::MAJOR_PROTOCOL_VERSION);

            let length_in_bytes_of_body: u32 = u32::try_from(body_serialized.len())?;
            v.extend_from_slice(&length_in_bytes_of_body.to_be_bytes());

            v.extend_from_slice(&serial_of_this_message.to_be_bytes());

            // A header field is an Array of Struct of (Byte, Variant).
            for header_field in header_fields.vec {
                v.extend_from_slice(&header_field.marshall_be()?);
            }

            v
        };

        // Header must be 8-aligned with null bytes
        while header_serialized.len() % 8 > 0 {
            header_serialized.push(0);
        }

        self.writer.write_all(&header_serialized).await?;
        self.writer.flush().await?;

        loop {
            let mut buf = vec![0; 1024];
            let n = self.reader.read(&mut buf).await?;
            if n > 0 {
                dbg!(&buf[..n]);
            }
        }

        Ok(())
    }

    // async fn send_recv(&mut self, payload: &[u8]) -> crate::Result<()> {
    //     if let Ok(c) = std::str::from_utf8(payload) {
    //         println!("C: {}", c);
    //     }

    //     // self.stream.write_all(payload).await?;
    //     self.writer.write_all(payload).await?;

    //     let mut buf = vec![0u8; 1024];

    //     let n = self.reader.read(&mut buf).await?;
    //     if n > 0 {
    //         if let Ok(s) = std::str::from_utf8(&buf[..n]) {
    //             println!("S: {}", s);
    //         }
    //     }

    //     Ok(())
    // }

    async fn write_line<T: AsRef<str>>(&mut self, line: T) -> crate::Result<()> {
        let line: &str = line.as_ref();

        log::debug!("C: {}", line);

        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\r\n").await?;
        self.writer.flush().await?;

        Ok(())
    }

    async fn auth(&mut self) -> crate::Result<()> {
        // Send AUTH EXTERNAL with UID
        self.write_line(format!("AUTH EXTERNAL {}", get_uid_as_hex()))
            .await?;

        // Expect to get OK from server
        let line: String = self.read_line().await?;
        if !line.starts_with("OK") {
            return Err(crate::Error::FailedAuth);
        }

        // Send BEGIN command
        self.write_line("BEGIN").await?;

        Ok(())
    }

    async fn read_line(&mut self) -> crate::Result<String> {
        let mut line: String = String::new();

        self.reader.read_line(&mut line).await;
        debug_assert!(line.ends_with('\n'));

        // In DBus, \r\n indicates a line ending, but messages are not expected to
        // span multiple lines.
        assert!(line.ends_with("\r\n"));

        // Pop the trailing "\r\n" from the line.
        line.pop();
        line.pop();

        log::debug!("S: {}", line);

        Ok(line)
    }

    /// Spec requires us to say hello.
    async fn say_hello(&mut self) -> crate::Result<()> {
        let destination = DBusString {
            string: String::from("org.freedesktop.DBus"),
        };

        let object_path = DBusObjectPath {
            dbus_string: DBusString {
                string: String::from("org/freedesktop/DBus"),
            },
        };

        let interface = DBusString {
            string: String::from("org.freedesktop.DBus"),
        };

        let member = DBusString {
            string: String::from("Hello"),
        };

        let body = Body {};

        self.call_method(destination, object_path, interface, member, body)
            .await?;

        Ok(())
    }
}
