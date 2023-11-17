use std::{borrow::Borrow, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

macro_rules! impl_id {
    ($name:ident) => {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Arc<str>);

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T> From<T> for $name
        where
            T: Borrow<str>,
        {
            fn from(value: T) -> Self {
                Self(value.borrow().into())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

impl_id!(UserId);
impl_id!(DeviceId);
impl_id!(VideoId);
