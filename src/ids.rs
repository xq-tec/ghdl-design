//! Design table ID types.

use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroU32;

use serde::Deserialize;
use serde::Deserializer;

/// Primitive for design table IDs.
pub type IdPrimitive = NonZeroU32;

macro_rules! design_id {
    ($($name:ident),+ $(,)?) => {
        $(
            /// Design table row ID (1-based on the wire, use [`Self::index`] for array access).
            #[derive(Clone, Copy)]
            pub struct $name(IdPrimitive);

            impl $name {
                /// Creates an ID from a non-zero raw index.
                #[must_use]
                pub const fn from_raw(id: IdPrimitive) -> Self {
                    Self(id)
                }

                /// Returns the non-zero wire index.
                #[must_use]
                pub fn to_raw(self) -> IdPrimitive {
                    self.0
                }

                /// Returns the 0-based index into the corresponding packed table (`raw - 1`).
                #[must_use]
                pub fn index(self) -> usize {
                    self.0.get() as usize - 1
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    let id = IdPrimitive::deserialize(deserializer)?;
                    Ok(Self(id))
                }
            }

            impl From<IdPrimitive> for $name {
                fn from(id: IdPrimitive) -> Self {
                    Self(id)
                }
            }

            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }

            impl Eq for $name {}

            impl Hash for $name {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.0.hash(state);
                }
            }

            impl fmt::Debug for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "{}({})", stringify!($name), self.0.get())
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Display::fmt(&self.0.get(), formatter)
                }
            }
        )+
    };
}

design_id!(
    InstanceId,
    SignalId,
    ProcessId,
    DriverId,
    SensitivityId,
    ConnectionId,
    DisconnectId,
    TypeId,
    ValueId,
    MemoryId,
    NbrSourcesId,
);

/// Deserializes an optional design ID where `0` means absent.
///
/// # Errors
///
/// Returns an error if deserialization to `u32` fails.
pub fn deserialize_optional_id<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: From<IdPrimitive>,
{
    let id = u32::deserialize(deserializer)?;
    Ok(IdPrimitive::new(id).map(T::from))
}
