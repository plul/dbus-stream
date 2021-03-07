use std::collections::HashSet;
use std::convert::TryFrom;

use smol::io::BufReader;
use smol::io::BufWriter;
use smol::prelude::*;

use crate::message_protocol::body::Body;
use crate::message_protocol::header::header_field;
use crate::message_protocol::header::header_field::HeaderField;
use crate::message_protocol::header::Header;
use crate::message_protocol::header::HeaderFlag;
use crate::message_protocol::MessageType;
use crate::message_protocol::MethodCall;
use crate::type_system::types::*;
use crate::type_system::*;

pub struct Connection {
    serial: u32,
    reader: BufReader<Box<dyn AsyncRead + Unpin>>,
    writer: BufWriter<Box<dyn AsyncWrite + Unpin>>,
}

impl Connection {
    pub async fn new_system() -> crate::Result<Self> {
        log::info!("Connecting to system DBus.");

        let mut conn = Self::connect_to_system_bus().await?;

        // Spec for some reason requires that the first thing we do is to send a null byte.
        log::debug!("Writing null byte.");
        conn.writer.write(&[0]).await?;

        log::info!("Authenticating.");
        conn.auth().await?;

        log::debug!("Saying hello.");
        conn.say_hello().await?;

        Ok(conn)
    }

    fn new<R, W>(reader: R, writer: W) -> Self
    where
        R: AsyncRead + Unpin + 'static,
        W: AsyncWrite + Unpin + 'static,
    {
        Connection {
            reader: BufReader::new(Box::new(reader)),
            writer: BufWriter::new(Box::new(writer)),
            serial: 0,
        }
    }

    #[cfg(windows)]
    async fn connect_to_system_bus() -> crate::Result<Connection> {
        use smol::net::TcpStream;

        let address = todo!("address of DBus system bus on Windows in the format of ip and port number, for example: 127.0.0.1:8080");

        let stream = TcpStream::connect(address).await?;

        // Split up into buffered read/write.
        let reader = stream.clone();
        let writer = stream;

        let conn = Self::new(reader, writer);

        Ok(conn)
    }

    #[cfg(unix)]
    async fn connect_to_system_bus() -> crate::Result<Connection> {
        use smol::net::unix::UnixStream;

        // TODO check DBUS_SYSTEM_BUS_ADDRESS env variable, if it is set, connect to that instead.
        let address = "/var/run/dbus/system_bus_socket";

        let stream = UnixStream::connect(address).await?;

        // Split up into buffered read/write.
        let reader = stream.clone();
        let writer = stream;

        let conn = Self::new(reader, writer);

        Ok(conn)
    }

    pub fn marshall_message(
        &mut self,
        message_type: MessageType,
        flags: HashSet<HeaderFlag>,
        header_fields: Vec<HeaderField>,
        body: Body,
    ) -> crate::Result<Vec<u8>> {
        let marshalled_body: Vec<u8> = body.marshall_be()?;

        self.serial += 1;
        let serial = self.serial;

        let header: Header = Header {
            endianness: Endianness::BigEndian,
            message_type,
            flags,
            major_protocol_version: crate::MAJOR_PROTOCOL_VERSION,
            length_in_bytes_of_message_body: u32::try_from(marshalled_body.len())?,
            serial,
            header_fields,
        };
        let marshalled_header: Vec<u8> = header.marshall();

        // Header must be 8-aligned, but that is currently done in the marshall method of the header itself.
        debug_assert_eq!(marshalled_header.len() % 8, 0);

        let mut message: Vec<u8> = Vec::new();
        message.extend(marshalled_header);
        message.extend(marshalled_body);

        Ok(message)
    }

    /// Send marshalled message.
    async fn send_message(&mut self, message: &[u8]) -> crate::Result<()> {
        self.writer.write_all(&message).await?;
        self.writer.flush().await?;

        Ok(())
    }

    async fn call_method(&mut self, method_call: MethodCall) -> crate::Result<()> {
        let mut flags: HashSet<HeaderFlag> = HashSet::new();
        flags.insert(HeaderFlag::NoReplyExpected);

        let mut header_fields: Vec<HeaderField> = vec![
            HeaderField::Path(method_call.path),
            HeaderField::Destination(method_call.destination),
            HeaderField::Member(method_call.member),
        ];
        if let Some(interface) = method_call.interface {
            header_fields.push(HeaderField::Interface(interface));
        }

        todo!("body signature should be added as a header field. but it kinda requires it to be marshalled first");
        // HeaderField::Signature(method_call.body.signature()),

        let message: Vec<u8> = self.marshall_message(
            MessageType::MethodCall,
            flags,
            header_fields,
            method_call.body,
        )?;

        self.send_message(&message).await?;

        Ok(())
    }

    pub async fn call_method_expect_reply(&mut self, method_call: MethodCall) -> crate::Result<()> {
        self.call_method(method_call).await?;
        todo!("not sure what the return type of this will be");
        Ok(())
    }

    pub async fn call_method_no_reply(&mut self, method_call: MethodCall) -> crate::Result<()> {
        self.call_method(method_call).await?;
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

    /// Spec requires us to say hello.
    async fn say_hello(&mut self) -> crate::Result<()> {
        let destination = header_field::Destination {
            dbus_string: DBusString {
                string: String::from("org.freedesktop.DBus"),
            },
        };

        let path = header_field::Path {
            dbus_object_path: DBusObjectPath {
                dbus_string: DBusString {
                    string: String::from("org/freedesktop/DBus"),
                },
            },
        };

        let interface = header_field::Interface {
            dbus_string: DBusString {
                string: String::from("org.freedesktop.DBus"),
            },
        };

        let member = header_field::Member {
            dbus_string: DBusString {
                string: String::from("Hello"),
            },
        };

        let body = Body { arguments: vec![] };

        let method_call = MethodCall {
            destination,
            path,
            interface: Some(interface),
            member,
            body,
        };

        let reply = self.call_method_expect_reply(method_call).await?;
        todo!("What to do with the reply");

        Ok(())
    }

    /// Get AUTH EXTERNAL parameter for unix: UID as hex.
    #[cfg(unix)]
    fn get_auth_external_param() -> String {
        // Get UID
        let uid: nix::unistd::Uid = nix::unistd::getuid();
        let uid: u32 = uid.as_raw();
        // Convert it to a string, "1000" for example.
        let uid: String = format!("{}", uid);
        // Encode the "1000" string as lowercase hex, fx "31303030", which is the format
        // that the DBus auth protocol wants.
        let uid: String = hex::encode(uid);

        uid
    }

    /// Get AUTH EXTERNAL parameter for windows: SID as hex.
    #[cfg(windows)]
    fn get_auth_external_param() -> String {
        todo!("Dani: Here, return Windows SID as hex for use with AUTH. Take a look at get_uid_as_hex for inspiration");
    }

    /// Authenticate with the DBus.
    async fn auth(&mut self) -> crate::Result<()> {
        // Send AUTH EXTERNAL
        self.auth_write_line(format!("AUTH EXTERNAL {}", Self::get_auth_external_param()))
            .await?;

        // Expect to get OK from server
        let line: String = self.auth_read_line().await?;
        if !line.starts_with("OK") {
            return Err(crate::Error::FailedAuth);
        }

        // Send BEGIN command
        self.auth_write_line("BEGIN").await?;

        Ok(())
    }

    /// Write one line to the stream, delimited by \r\n.
    ///
    /// The line ending \r\n is added inside this method, so should not be included in the
    /// input.
    ///
    /// This is only used for the AUTH protocol, which is line based.
    async fn auth_write_line<T: AsRef<str>>(&mut self, line: T) -> crate::Result<()> {
        let line: &str = line.as_ref();

        log::debug!("C: {}", line);

        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\r\n").await?;
        self.writer.flush().await?;

        Ok(())
    }

    /// Read one line from the stream, delimited by \r\n.
    ///
    /// The returned string does not contain the trailing \r\n.
    ///
    /// This is used only for the AUTH protocol, which is line based.
    async fn auth_read_line(&mut self) -> crate::Result<String> {
        let mut line: String = String::new();

        self.reader.read_line(&mut line).await?;
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
}
