//! Transient ordered records over one durable canonical state slot.
use crate::{Error, Record, RecordUpgrade, Result, record_codec};
use std::marker::PhantomData;

const PREFIX: &str = "record-list1-";
const SCHEMA_NAME_LEN: usize = PREFIX.len() + 16;

struct Schema<T>(PhantomData<T>);
impl<T: Record> Schema<T> {
    const NAME: [u8; SCHEMA_NAME_LEN] = record_codec::schema_name(
        PREFIX,
        record_codec::nested_fingerprint(0xcbf29ce484222325, &[T::FORMAT]),
    );
    const PREVIOUS_NAME: [u8; SCHEMA_NAME_LEN] = record_codec::schema_name(
        PREFIX,
        record_codec::nested_fingerprint(
            0xcbf29ce484222325,
            &[match T::PREVIOUS_FORMAT {
                Some(name) => name,
                None => "",
            }],
        ),
    );
}
impl<T: Record> Record for Vec<T> {
    const FORMAT: &'static str = match std::str::from_utf8(&Schema::<T>::NAME) {
        Ok(name) => name,
        Err(_) => panic!("Invalid record list schema"),
    };
    const PREVIOUS_FORMAT: Option<&'static str> = match T::PREVIOUS_FORMAT {
        Some(_) => match std::str::from_utf8(&Schema::<T>::PREVIOUS_NAME) {
            Ok(name) => Some(name),
            Err(_) => panic!("Invalid previous record list schema"),
        },
        None => None,
    };
    fn encode(&self) -> Result<String> {
        let mut output = String::new();
        record_codec::append_records(&mut output, self)?;
        Ok(output)
    }
    fn decode(payload: &str) -> Result<Self> {
        let mut reader = record_codec::Reader::new(payload)?;
        let records = reader.records()?;
        reader.finish()?;
        Ok(records)
    }
    fn upgrade_encoded(encoded: &str) -> Result<Self> {
        if encoded.len() > 1024 * 1024 {
            return Err(Error::Application("Record payload exceeds 1 MiB".into()));
        }
        let (_, payload) = encoded
            .split_once('\n')
            .filter(|(format, _)| Some(*format) == Self::PREVIOUS_FORMAT)
            .ok_or_else(|| Error::Application("Record list format requires migration".into()))?;
        let mut reader = record_codec::Reader::new(payload)?;
        let records = reader.upgrade_records::<T>()?;
        reader.finish()?;
        Ok(records)
    }
}

impl<T, Previous> RecordUpgrade<Vec<Previous>> for Vec<T>
where
    T: RecordUpgrade<Previous>,
    Previous: Record,
{
    fn upgrade(previous: Vec<Previous>) -> Result<Self> {
        previous.into_iter().map(T::upgrade).collect()
    }
}
