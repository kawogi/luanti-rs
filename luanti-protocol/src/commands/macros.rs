#[macro_export]
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}

// #[macro_export]
// macro_rules! default_serializer {
//     ($spec_ty: ident { }) => {
//         impl Serialize for $spec_ty {
//             type Input = Self;
//             fn serialize<S: Serializer>(value: &Self::Input, _: &mut S) -> SerializeResult {
//                 Ok(())
//             }
//         }
//     };
//     ($spec_ty: ident { $($fname: ident: $ftyp: ty ),+ }) => {
//         impl Serialize for $spec_ty {
//             type Input = Self;
//             fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
//                 $(
//                     <$ftyp as Serialize>::serialize(&value.$fname, ser)?;
//                 )+
//                 Ok(())
//             }
//         }
//     };
// }

// #[macro_export]
// macro_rules! default_deserializer {
//     ($spec_ty: ident { }) => {
//         impl Deserialize for $spec_ty {
//             type Output = Self;
//             fn deserialize(_deserializer: &mut Deserializer) -> DeserializeResult<Self> {
//                 log::trace!(stringify!("deserializing ", $spec_ty));
//                 Ok($spec_ty)
//             }
//         }
//     };
//     ($spec_ty: ident { $($fname: ident: $ftyp: ty ),+ }) => {
//         impl Deserialize for $spec_ty {
//             type Output = Self;
//             fn deserialize(deserializer: &mut Deserializer) -> DeserializeResult<Self> {
//                 log::trace!(stringify!("deserializing ", $spec_ty));
//                 $(
//                     log::trace!(stringify!("deserializing field ", $fname, ": ", $ftyp));
//                     let $fname = <$ftyp>::deserialize(deser)?;
//                 )+
//                 Ok($spec_ty { $($fname, )+ })
//             }
//         }
//     };
// }

#[macro_export]
macro_rules! implicit_from {
    ($command_ty: ident, $name: ident, $spec_ty: ident) => {
        impl From<$spec_ty> for $command_ty {
            fn from(value: $spec_ty) -> Self {
                $command_ty::$name(Box::new(value))
            }
        }
    };
}

macro_rules! define_protocol {
    ($version: literal,
     $protocol_id: literal,
     $dir: ident,
     $command_ty: ident => {
         $($name: ident, $id: literal, $channel: ident, $reliable: literal => $spec_ty: ident),*
    }) => {
        $crate::as_item! {
            #[derive(Debug, PartialEq, Clone)]
            pub enum $command_ty {
                $($name(Box<$spec_ty>)),*,
            }
        }

        $crate::as_item! {
            impl CommandProperties for $command_ty {
                fn direction(&self) -> CommandDirection {
                    CommandDirection::$dir
                }

                fn default_channel(&self) -> ChannelId {
                    match self {
                        $($command_ty::$name(_) => ChannelId::$channel),*,
                    }
                }

                fn default_reliability(&self) -> bool {
                    match self {
                        $($command_ty::$name(_) => $reliable),*,
                    }
                }

                fn command_name(&self) -> &'static str {
                    match self {
                        $($command_ty::$name(_) => stringify!($name)),*,
                    }
                }
            }
        }

        $crate::as_item! {
            impl Serialize for $command_ty {
                type Input = Self;
                fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
                    match value {
                        $($command_ty::$name(spec) => {
                            u16::serialize(&$id, ser)?;
                            anyhow::Context::context(<$spec_ty as Serialize>::serialize(Deref::deref(spec), ser), stringify!($name))
                        }),*,
                    }
                }
            }
        }

        $crate::as_item! {
            impl Deserialize for $command_ty {
                type Output = Option<Self>;
                fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
                    // The first packet a client sends doesn't contain a command but has an empty payload.
                    // It only serves the purpose of triggering the creation of a peer entry within the server.
                    // Rather than requesting every caller to perform a pre-check for a non-empty payload,
                    // we just return an `Option` to force the caller to handle this case.
                    if !deserializer.has_remaining() {
                        return Ok(None);
                    }
                    let orig_buffer = deserializer.peek_all();
                    // log::trace!("orig_buffer: {:?}", &orig_buffer[0..(orig_buffer.len().min(64))]);
                    let command_id = u16::deserialize(deserializer)?;
                    let dir = deserializer.direction();
                    let result = match (dir, command_id) {
                        $( (CommandDirection::$dir, $id) => $command_ty::$name(Box::new(<$spec_ty as Deserialize>::deserialize(deserializer)?)) ),*,
                        _ => bail!(DeserializeError::BadPacketId(dir, command_id)),
                    };
                    // there might be more bytes to read if new fields have been added to the protocol
                    // those will be stripped off and might trip the receiver
                    if deserializer.has_remaining() {
                        log::warn!("left-over bytes after deserialization of {:#?}: {:?}", result, deserializer.peek_all());
                    }
                    audit_command(deserializer.context(), orig_buffer, &result);
                    Ok(Some(result))
                }
            }
        }

        $($crate::implicit_from!($command_ty, $name, $spec_ty);)*
    };
}
