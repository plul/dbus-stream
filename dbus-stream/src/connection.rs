use std::collections::HashSet;
use std::convert::TryFrom;
use std::num::NonZeroU32;

use smol::io::BufReader;
use smol::io::BufWriter;
use smol::prelude::*;

use crate::message_protocol::body::Body;
use crate::message_protocol::Message;
use crate::message_protocol::MessageType;
use crate::message_protocol::MessageTypeParam;
use crate::message_protocol::MethodCall;
use crate::type_system::types::*;
use crate::type_system::*;

pub struct Connection {
    /// Serial that is unique for each message, so replies can be identified.
    /// Increment by one for each sent message to keep it unique.
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

    fn get_serial(&mut self) -> NonZeroU32 {
        // Increment
        self.serial += 1;

        NonZeroU32::new(self.serial).expect("Serial overflow")
    }

    /// Send marshalled message.
    async fn send_message(&mut self, message: &Message) -> crate::Result<()> {
        let marshalled = message.marshall_be()?;
        self.writer.write_all(&marshalled).await?;
        self.writer.flush().await?;
        Ok(())
    }

    /// DBus method call, with reply.
    pub async fn call_method_expect_reply(&mut self, message: &Message) -> crate::Result<()> {
        self.send_message(&message).await?;

        let mut buf = [1; 1];
        self.reader.read_exact(&mut buf).await?;
        dbg!(buf);
        todo!("Complete this. Read and unmarshall the whole message");

        Ok(())
    }

    fn formulate_message(
        &mut self,
        message_type_param: MessageTypeParam,
        destination: Option<DBusString>,
        body: Body,
    ) -> Message {
        let serial = self.get_serial();

        Message {
            flag_no_reply_expected: false,
            flag_no_auto_start: false,
            flag_allow_interactive_authorization: false,
            serial,
            message_type_param,
            destination,
            body,
        }
    }

    /// Spec requires us to say hello on new connections immediately after AUTH.
    async fn say_hello(&mut self) -> crate::Result<()> {
        let destination = DBusString::new("org.freedesktop.DBus")?;
        let path = DBusObjectPath::new("org/freedesktop/DBus")?;
        let interface = DBusString::new("org.freedesktop.DBus")?;
        let member = DBusString::new("Hello")?;

        let body = Body { arguments: vec![] };

        let method_call = MethodCall {
            path,
            interface: Some(interface),
            member,
        };

        let message = self.formulate_message(
            MessageTypeParam::MethodCall(method_call),
            Some(destination),
            body,
        );

        let reply = self.call_method_expect_reply(&message).await?;
        todo!("What to do with the reply?");

        Ok(())
    }

    /// Get AUTH EXTERNAL parameter for unix: UID as hex.
    #[cfg(unix)]
    fn get_auth_external_param() -> crate::Result<String> {
        // Get UID
        let uid: nix::unistd::Uid = nix::unistd::getuid();
        let uid: u32 = uid.as_raw();
        // Convert it to a string, "1000" for example.
        let uid: String = format!("{}", uid);
        // Encode the "1000" string as lowercase hex, fx "31303030", which is the format
        // that the DBus auth protocol wants.
        let uid: String = hex::encode(uid);

        Ok(uid)
    }

    /// Get AUTH EXTERNAL parameter for windows: SID as hex.
    #[cfg(windows)]
    fn get_auth_external_param() -> crate::Result<String> {
        // Get the user in order to get its assigned SID.
        let user: String = windows_acl::helper::current_user().unwrap();
        let sid: Vec<u8> = windows_acl::helper::name_to_sid(&user, None).unwrap();
        // Convert it to a string, "1000" for example.
        let sid: String = String::from_utf8(sid)?;
        // Encode the "1000" string as lowercase hex, fx "31303030", which is the format
        // that the DBus auth protocol wants.
        let sid: String = hex::encode(sid);

        Ok(sid)
    }

    /// Authenticate with the DBus.
    async fn auth(&mut self) -> crate::Result<()> {
        // Send AUTH EXTERNAL
        self.auth_write_line(format!(
            "AUTH EXTERNAL {}",
            Self::get_auth_external_param()?
        ))
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
