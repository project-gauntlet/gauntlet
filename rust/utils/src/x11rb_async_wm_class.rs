use x11rb_async::Cookie;
use x11rb_async::connection::Connection;
use x11rb_async::errors::ConnectionError;
use x11rb_async::errors::ParseError;
use x11rb_async::errors::ReplyError;
use x11rb_async::protocol::xproto::AtomEnum;
use x11rb_async::protocol::xproto::GetPropertyReply;
use x11rb_async::protocol::xproto::Window;
use x11rb_async::protocol::xproto::{self};

macro_rules! property_cookie {
    {
        $(#[$meta:meta])*
        pub struct $cookie_name:ident: $struct_name:ident,
        $from_reply:expr,
    } => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $cookie_name<'a, Conn: Connection + ?Sized>(Cookie<'a, Conn, GetPropertyReply>);

        impl<'a, Conn> $cookie_name<'a, Conn>
        where
            Conn: Connection + ?Sized,
        {
            /// Get the reply that the server sent.
            pub async fn reply(self) -> Result<Option<$struct_name>, ReplyError> {
                #[allow(clippy::redundant_closure_call)]
                Ok($from_reply(self.0.reply().await?)?)
            }

            /// Get the reply that the server sent, but have errors handled as events.
            pub async fn reply_unchecked(self) -> Result<Option<$struct_name>, ConnectionError> {
                self.0
                    .reply_unchecked().await?
                    .map($from_reply)
                    .transpose()
                    .map(|e| e.flatten())
                    .map_err(Into::into)
            }
        }
    }
}

// WM_CLASS

property_cookie! {
    /// A cookie for getting a window's `WM_CLASS` property.
    ///
    /// See `WmClass`.
    pub struct WmClassCookie: WmClass,
    WmClass::from_reply,
}

impl<'a, Conn> WmClassCookie<'a, Conn>
where
    Conn: Connection + ?Sized,
{
    /// Send a `GetProperty` request for the `WM_CLASS` property of the given window
    pub async fn new(conn: &'a Conn, window: Window) -> Result<Self, ConnectionError> {
        Ok(Self(
            xproto::get_property(conn, false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 2048).await?,
        ))
    }
}

#[derive(Debug)]
pub struct WmClass(GetPropertyReply, usize);

impl WmClass {
    /// Send a `GetProperty` request for the `WM_CLASS` property of the given window
    pub async fn get<C: Connection>(conn: &C, window: Window) -> Result<WmClassCookie<'_, C>, ConnectionError> {
        WmClassCookie::new(conn, window).await
    }

    /// Construct a new `WmClass` instance from a `GetPropertyReply`.
    ///
    /// The original `GetProperty` request must have been for a `WM_CLASS` property for this
    /// function to return sensible results.
    pub fn from_reply(reply: GetPropertyReply) -> Result<Option<Self>, ParseError> {
        if reply.type_ == AtomEnum::NONE.into() {
            return Ok(None);
        }
        if reply.type_ != AtomEnum::STRING.into() || reply.format != 8 {
            return Err(ParseError::InvalidValue);
        }
        // Find the first zero byte in the value
        let offset = reply.value.iter().position(|&v| v == 0).unwrap_or(reply.value.len());
        Ok(Some(WmClass(reply, offset)))
    }

    /// Get the instance contained in this `WM_CLASS` property
    pub fn instance(&self) -> &[u8] {
        &self.0.value[0..self.1]
    }

    /// Get the class contained in this `WM_CLASS` property
    pub fn class(&self) -> &[u8] {
        let start = self.1 + 1;
        if start >= self.0.value.len() {
            return &[];
        };
        let end = if self.0.value.last() == Some(&0) {
            self.0.value.len() - 1
        } else {
            self.0.value.len()
        };
        &self.0.value[start..end]
    }
}
