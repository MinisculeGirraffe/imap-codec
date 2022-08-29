//! The IMAP QUOTA Extension
//!
//! This extension extends ...
//!
//! * the [Capability](crate::response::Capability::Quota) enum with a new variant:
//!
//!     - [Capability::Quota](crate::response::Capability#variant.Quota)
//!
//! * the [CommandBody](crate::command::CommandBody) enum with a new varients:
//!
//!     - [Command::SetQuota](crate::command::CommandBody::SetQuota)
//!     - [Command::GetQuota](crate::command::CommandBody::GetQuota)
//!     - [Command::GetQuotaRoot](crate::command::CommandBody::GetQuotaRoot)
//!
//! * the [Data](crate::response::Data) enum with new varients:
//!
//!     - [Data::Quota](crate::response::Data::Quota)
//!     - [Data::QuotaRoot](crate::response::Data::QuotaRoot)
//!

pub use crate::core::AString;
use crate::{codec::Encode, rfc3501::core::Atom};
#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetQuotaResource<'a> {
    pub name: QuotaResourceName<'a>,
    pub limit: u64,
}

impl<'a> Encode for SetQuotaResource<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.name.encode(writer)?;
        write!(writer, " {}", self.limit)
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotaResource<'a> {
    pub name: QuotaResourceName<'a>,
    pub usage: u64,
    pub limit: u64,
}

impl<'a> Encode for QuotaResource<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.name.encode(writer)?;
        write!(writer, " {} {}", self.usage, self.limit)
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuotaResourceName<'a> {
    /// Sum of messages' [RFC822](https://www.ietf.org/rfc/rfc822.txt).SIZE, in units of 1024 octets
    /// [RFC822](https://www.ietf.org/rfc/rfc822.txt).SIZE is the number of octets in the entire message, including message headers.
    Storage,
    /// Number of messages
    Message,
    Atom(Atom<'a>),
}

impl<'a> Encode for QuotaResourceName<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            QuotaResourceName::Storage => writer.write_all(b"STORAGE"),
            QuotaResourceName::Message => writer.write_all(b"MESSAGE"),
            QuotaResourceName::Atom(atom) => atom.encode(writer),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::{codec::Encode, response::Data, rfc3501::command::CommandBody};

    use super::{QuotaResource, QuotaResourceName, SetQuotaResource};

    fn compare_output(items: Vec<(Result<impl Encode, ()>, &str)>) {
        let mut writer: Vec<u8> = Vec::new();
        for item in items {
            item.0.unwrap().encode(&mut writer).unwrap();
            assert_eq!(std::str::from_utf8(&writer).unwrap(), item.1);
            writer.clear();
        }
    }

    #[test]
    fn command_output() {
        let commands = vec![
            (CommandBody::get_quota("INBOX"), "GETQUOTA INBOX"),
            (CommandBody::get_quota(""), "GETQUOTA \"\""),
            (
                CommandBody::get_quota_root("MAILBOX"),
                "GETQUOTAROOT MAILBOX",
            ),
            (CommandBody::set_quota("INBOX", vec![]), "SETQUOTA INBOX ()"),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![SetQuotaResource {
                        name: QuotaResourceName::Storage,
                        limit: 256,
                    }],
                ),
                "SETQUOTA INBOX (STORAGE 256)",
            ),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![
                        SetQuotaResource {
                            name: QuotaResourceName::Message,
                            limit: 256,
                        },
                        SetQuotaResource {
                            name: QuotaResourceName::Storage,
                            limit: 512,
                        },
                    ],
                ),
                "SETQUOTA INBOX (MESSAGE 256 STORAGE 512)",
            ),
        ];

        compare_output(commands)
    }
    #[test]
    fn response_output() {
        let responses = vec![
            (
                Data::quota(
                    "INBOX",
                    vec![QuotaResource {
                        name: QuotaResourceName::Message,
                        usage: 1024,
                        limit: 2048,
                    }],
                ),
                "* QUOTA INBOX (MESSAGE 1024 2048)\r\n",
            ),
            (
                Data::quota_root("INBOX", vec![]),
                "* QUOTAROOT INBOX \"\"\r\n",
            ),
            (
                Data::quota_root(
                    "INBOX",
                    vec!["ROOT1".try_into().unwrap(), "ROOT2".try_into().unwrap()],
                ),
                "* QUOTAROOT INBOX ROOT1 ROOT2\r\n",
            ),
        ];

        compare_output(responses)
    }
}
