//! Durable async result state; futures and completion scheduling belong to the host.
use crate::{Error, Record, Result, record_codec};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i64)]
pub enum AsyncPhase {
    #[default]
    Idle = 0,
    Pending = 1,
    Ready = 2,
    Failed = 3,
}

/// A strict async read is unavailable until the current request is ready.
/// This is data for a UI boundary, not a runtime transaction failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsyncReadError {
    Idle,
    Pending,
    Failed(String),
}

/// A typed current-generation read. Standard Result combinators and `?` skip
/// derived work while unavailable. Record computed nodes cache this as data;
/// runtime failures remain in an outer blockui::Result.
pub type AsyncResult<T> = std::result::Result<T, AsyncReadError>;

/// Combine independently read results without short-circuiting input reads.
pub trait AsyncResultExt<T> {
    /// Both arguments are evaluated before combining. Failed takes priority
    /// over Pending, which takes priority over Idle. If both fail, the left
    /// failure wins. This combines results; it does not start host work.
    fn join<U>(self, other: AsyncResult<U>) -> AsyncResult<(T, U)>;
}
impl<T> AsyncResultExt<T> for AsyncResult<T> {
    fn join<U>(self, other: AsyncResult<U>) -> AsyncResult<(T, U)> {
        match (self, other) {
            (Err(error @ AsyncReadError::Failed(_)), _) => Err(error),
            (_, Err(error @ AsyncReadError::Failed(_))) => Err(error),
            (Err(AsyncReadError::Pending), _) | (_, Err(AsyncReadError::Pending)) => {
                Err(AsyncReadError::Pending)
            }
            (Err(AsyncReadError::Idle), _) | (_, Err(AsyncReadError::Idle)) => {
                Err(AsyncReadError::Idle)
            }
            (Ok(left), Ok(right)) => Ok((left, right)),
        }
    }
}

struct ReadSchema<T>(PhantomData<T>);
impl<T: Record> ReadSchema<T> {
    const NAME: [u8; 14 + 16] = record_codec::schema_name(
        "async-result1-",
        record_codec::nested_fingerprint(0xcbf29ce484222325, &[T::FORMAT]),
    );
}
impl<T: Record> Record for AsyncResult<T> {
    const FORMAT: &'static str = match std::str::from_utf8(&ReadSchema::<T>::NAME) {
        Ok(name) => name,
        Err(_) => panic!("Invalid async result schema"),
    };
    fn encode(&self) -> Result<String> {
        let mut output = String::new();
        match self {
            Err(AsyncReadError::Idle) => record_codec::append(&mut output, &0_i64)?,
            Err(AsyncReadError::Pending) => record_codec::append(&mut output, &1_i64)?,
            Ok(value) => {
                record_codec::append(&mut output, &2_i64)?;
                record_codec::append_record(&mut output, value)?;
            }
            Err(AsyncReadError::Failed(message)) => {
                record_codec::append(&mut output, &3_i64)?;
                record_codec::append(&mut output, message)?;
            }
        }
        Ok(output)
    }
    fn decode(payload: &str) -> Result<Self> {
        let mut reader = record_codec::Reader::new(payload)?;
        let value = match reader.field::<i64>()? {
            0 => Err(AsyncReadError::Idle),
            1 => Err(AsyncReadError::Pending),
            2 => Ok(reader.record()?),
            3 => Err(AsyncReadError::Failed(reader.field()?)),
            _ => return Err(Error::InvalidView("invalid async result phase")),
        };
        reader.finish()?;
        Ok(value)
    }
}

/// A durable async generation and latest result. Publish through State so reads
/// participate in the existing reactive graph. A pending refresh retains latest.
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncValue<T> {
    generation: i64,
    phase: AsyncPhase,
    latest: Option<T>,
    error: Option<String>,
}
impl<T> Default for AsyncValue<T> {
    fn default() -> Self {
        Self {
            generation: 0,
            phase: AsyncPhase::Idle,
            latest: None,
            error: None,
        }
    }
}
impl<T> AsyncValue<T> {
    /// Current request generation, also usable to retire a pending request.
    pub fn generation(&self) -> i64 {
        self.generation
    }
    pub fn phase(&self) -> AsyncPhase {
        self.phase
    }
    pub fn is_pending(&self) -> bool {
        self.phase == AsyncPhase::Pending
    }
    pub fn latest(&self) -> Option<&T> {
        self.latest.as_ref()
    }
    /// Read the current result. Unlike `latest`, a refresh yields Pending even
    /// when an earlier successful value is retained. Reads of the enclosing
    /// State establish the reactive dependency before this method is called.
    pub fn read(&self) -> std::result::Result<&T, AsyncReadError> {
        match self.phase {
            AsyncPhase::Idle => Err(AsyncReadError::Idle),
            AsyncPhase::Pending => Err(AsyncReadError::Pending),
            AsyncPhase::Failed => Err(AsyncReadError::Failed(
                self.error.clone().expect("validated async failure"),
            )),
            AsyncPhase::Ready => Ok(self.latest.as_ref().expect("validated async result")),
        }
    }
    /// Consume a decoded snapshot without cloning its current value or failure.
    /// Any retained stale value is discarded from this snapshot, not from State.
    pub fn into_read(self) -> AsyncResult<T> {
        match self.phase {
            AsyncPhase::Idle => Err(AsyncReadError::Idle),
            AsyncPhase::Pending => Err(AsyncReadError::Pending),
            AsyncPhase::Failed => Err(AsyncReadError::Failed(
                self.error.expect("validated async failure"),
            )),
            AsyncPhase::Ready => Ok(self.latest.expect("validated async result")),
        }
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    /// Begin/replace a request. Carry the returned scalar in the completion message.
    pub fn begin(&mut self) -> Result<i64> {
        let generation = self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.generation = generation;
        self.phase = AsyncPhase::Pending;
        self.error = None;
        Ok(generation)
    }
    /// Publish only the currently pending generation; stale/duplicate replies are ignored.
    pub fn resolve(&mut self, generation: i64, value: T) -> bool {
        if generation != self.generation || !self.is_pending() {
            return false;
        }
        self.latest = Some(value);
        self.error = None;
        self.phase = AsyncPhase::Ready;
        true
    }
    /// Keep any latest value available while exposing the current failure.
    pub fn reject(&mut self, generation: i64, error: String) -> bool {
        if generation != self.generation || !self.is_pending() {
            return false;
        }
        self.error = Some(error);
        self.phase = AsyncPhase::Failed;
        true
    }
    /// Retire the generation even if its completion is already in the mailbox.
    /// Cancel the host Task/Request separately to retire work and external resources.
    pub fn cancel(&mut self, generation: i64) -> Result<bool> {
        if generation != self.generation || !self.is_pending() {
            return Ok(false);
        }
        self.generation = self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.phase = AsyncPhase::Idle;
        self.error = None;
        Ok(true)
    }
    fn validate(&self) -> Result<()> {
        if self.generation < 0
            || (self.phase != AsyncPhase::Idle && self.generation == 0)
            || (self.phase == AsyncPhase::Ready && self.latest.is_none())
            || (self.phase == AsyncPhase::Failed) != self.error.is_some()
        {
            return Err(Error::InvalidView("invalid async value state"));
        }
        Ok(())
    }
}
const PREFIX: &str = "async-value1-";
struct Schema<T>(PhantomData<T>);
impl<T: Record> Schema<T> {
    const NAME: [u8; PREFIX.len() + 16] = record_codec::schema_name(
        PREFIX,
        record_codec::nested_fingerprint(0xcbf29ce484222325, &[T::FORMAT]),
    );
}
impl<T: Record> Record for AsyncValue<T> {
    const FORMAT: &'static str = match std::str::from_utf8(&Schema::<T>::NAME) {
        Ok(name) => name,
        Err(_) => panic!("Invalid async value schema"),
    };
    fn encode(&self) -> Result<String> {
        self.validate()?;
        let mut output = String::new();
        record_codec::append(&mut output, &self.generation)?;
        record_codec::append(&mut output, &(self.phase as i64))?;
        record_codec::append_optional_record(&mut output, &self.latest)?;
        record_codec::append(&mut output, &self.error)?;
        Ok(output)
    }
    fn decode(payload: &str) -> Result<Self> {
        let mut reader = record_codec::Reader::new(payload)?;
        let generation = reader.field()?;
        let phase = match reader.field::<i64>()? {
            0 => AsyncPhase::Idle,
            1 => AsyncPhase::Pending,
            2 => AsyncPhase::Ready,
            3 => AsyncPhase::Failed,
            _ => return Err(Error::InvalidView("invalid async phase")),
        };
        let value = Self {
            generation,
            phase,
            latest: reader.optional_record()?,
            error: reader.field()?,
        };
        reader.finish()?;
        value.validate()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Data;
    impl Record for Data {
        const FORMAT: &'static str = "async-test-data1";
        fn encode(&self) -> Result<String> {
            Ok(String::new())
        }
        fn decode(payload: &str) -> Result<Self> {
            if payload.is_empty() {
                Ok(Self)
            } else {
                Err(Error::InvalidView("invalid test data"))
            }
        }
    }
    #[test]
    fn generation_overflow_preserves_current_state() {
        let mut value = AsyncValue::<Data> {
            generation: i64::MAX,
            phase: AsyncPhase::Pending,
            latest: None,
            error: None,
        };
        assert!(value.begin().is_err());
        assert!(value.cancel(i64::MAX).is_err());
        assert_eq!(value.generation(), i64::MAX);
        assert!(value.is_pending());
    }
    #[test]
    fn malformed_phase_and_record_invariants_are_rejected() {
        let payload = |generation: i64, phase: i64, error: Option<String>| {
            let mut output = String::new();
            record_codec::append(&mut output, &generation).unwrap();
            record_codec::append(&mut output, &phase).unwrap();
            record_codec::append_optional_record::<Data>(&mut output, &None).unwrap();
            record_codec::append(&mut output, &error).unwrap();
            output
        };
        for (generation, phase, error) in [
            (0, 1, None),
            (-1, 0, None),
            (1, 9, None),
            (1, 2, None),
            (1, 3, None),
            (1, 1, Some("bad".into())),
        ] {
            assert!(AsyncValue::<Data>::decode(&payload(generation, phase, error)).is_err());
        }
        let valid = payload(1, 3, Some(String::new()));
        assert_eq!(
            AsyncValue::<Data>::decode(&valid).unwrap().error(),
            Some("")
        );
        assert!(AsyncValue::<Data>::decode(&(valid + "extra")).is_err());
    }
}
