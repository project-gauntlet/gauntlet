pub use x11rb_protocol::x11_utils::BigRequests;
pub use x11rb_protocol::x11_utils::ExtInfoProvider;
pub use x11rb_protocol::x11_utils::ExtensionInformation;
pub use x11rb_protocol::x11_utils::ReplyParsingFunction;
pub use x11rb_protocol::x11_utils::Request;
pub use x11rb_protocol::x11_utils::RequestHeader;
pub use x11rb_protocol::x11_utils::Serialize;
pub use x11rb_protocol::x11_utils::TryParse;
pub use x11rb_protocol::x11_utils::TryParseFd;
pub use x11rb_protocol::x11_utils::X11Error;
pub use x11rb_protocol::x11_utils::parse_request_header;

#[macro_export]
macro_rules! atom_manager {
    {
        $(#[$struct_meta:meta])*
        $vis:vis $struct_name:ident:
        $(#[$cookie_meta:meta])*
        $cookie_name:ident {
            $($field_name:ident$(: $atom_value:expr)?,)*
        }
    } => {
        // Cookie version
        #[allow(non_snake_case)]
        #[derive(Debug)]
        $(#[$cookie_meta])*
        $vis struct $cookie_name<'a, C: x11rb_async::connection::Connection> {
            __private_phantom: ::std::marker::PhantomData<&'a C>,
            __private_cookies: ::std::vec::Vec<x11rb_async::Cookie<'a, C, x11rb_async::protocol::xproto::InternAtomReply>>,
        }

        // Replies
        #[allow(non_snake_case)]
        #[derive(Debug, Clone, Copy)]
        $(#[$struct_meta])*
        $vis struct $struct_name {
            $(
                $vis $field_name: x11rb_async::protocol::xproto::Atom,
            )*
        }

        impl $struct_name {
            $vis async fn new<C: x11rb_async::connection::Connection>(
                _conn: &C,
            ) -> ::std::result::Result<$cookie_name<'_, C>, x11rb_async::errors::ConnectionError> {
                use futures::stream::{self, StreamExt, TryStreamExt};
                use x11rb_async::protocol::xproto::ConnectionExt;
                let names = futures::stream::iter(vec![
                    $($crate::__atom_manager_atom_value!($field_name$(: $atom_value)?),)*
                ]);
                let cookies: ::std::result::Result<::std::vec::Vec<_>, _> = names
                    .then(|name| _conn.intern_atom(false, name))
                    .try_collect()
                    .await;
                Ok($cookie_name {
                    __private_phantom: ::std::marker::PhantomData,
                    __private_cookies: cookies?,
                })
            }
        }

        impl<'a, C: x11rb_async::connection::Connection> $cookie_name<'a, C> {
            $vis async fn reply(self) -> ::std::result::Result<$struct_name, x11rb_async::errors::ReplyError> {
                let mut replies = self.__private_cookies.into_iter();
                Ok($struct_name {
                    $(
                        $field_name: replies.next().expect("new() should have constructed a Vec of the correct size").reply().await?.atom,
                    )*
                })
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __atom_manager_atom_value {
    ($field_name:ident) => {
        stringify!($field_name).as_bytes()
    };
    ($field_name:ident: $atom_value:expr) => {
        $atom_value
    };
}
