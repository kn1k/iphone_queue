#![allow(bare_trait_objects)]
#![allow(renamed_and_removed_lints)]

pub use self::iphone_queue::{ Participant, Add, Buy, Remove };

include!(concat!(env!("OUT_DIR"), "/protobuf_mod.rs"));

use exonum::proto::schema::*;
