//! Stateful native codecs. Decoder state contains no JS/context handles.
use encoding_rs::{Decoder, DecoderResult, Encoding, REPLACEMENT};
use rquickjs::{Ctx, Exception, JsLifetime, Result, TypedArray, class::Trace};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct NativeTextDecoderState {
    #[qjs(skip_trace)]
    encoding: &'static Encoding,
    #[qjs(skip_trace)]
    decoder: Decoder,
    fatal: bool,
    ignore_bom: bool,
}
#[rquickjs::methods]
impl NativeTextDecoderState {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, label: String, fatal: bool, ignore_bom: bool) -> Result<Self> {
        let encoding = Encoding::for_label(label.as_bytes())
            .filter(|e| *e != REPLACEMENT)
            .ok_or_else(|| Exception::throw_range(&ctx, "Unknown encoding"))?;
        Ok(Self {
            encoding,
            decoder: Self::decoder(encoding, ignore_bom),
            fatal,
            ignore_bom,
        })
    }
    #[qjs(skip)]
    fn decoder(encoding: &'static Encoding, ignore_bom: bool) -> Decoder {
        if ignore_bom {
            encoding.new_decoder_without_bom_handling()
        } else {
            encoding.new_decoder_with_bom_removal()
        }
    }
    #[qjs(get)]
    pub fn encoding(&self) -> String {
        self.encoding.name().to_ascii_lowercase()
    }
    pub fn decode(
        &mut self,
        ctx: Ctx<'_>,
        input: TypedArray<'_, u8>,
        stream: bool,
    ) -> Result<String> {
        // Decode synchronously from the live input view. The decoder and output
        // allocate only Rust memory; no JS call or await occurs while borrowed.
        let bytes = unsafe { input.as_bytes() }.ok_or(rquickjs::Error::Allocation)?;
        let mut output = String::new();
        let mut offset = 0;
        let mut invalid = false;
        loop {
            let capacity = self
                .decoder
                .max_utf8_buffer_length(bytes.len() - offset)
                .ok_or(rquickjs::Error::Allocation)?;
            output.reserve(capacity);
            if self.fatal {
                let (result, read) = self.decoder.decode_to_string_without_replacement(
                    &bytes[offset..],
                    &mut output,
                    !stream,
                );
                offset += read;
                match result {
                    DecoderResult::InputEmpty => break,
                    DecoderResult::OutputFull => continue,
                    DecoderResult::Malformed(_, _) => {
                        invalid = true;
                        break;
                    }
                }
            } else {
                let (result, read, _) =
                    self.decoder
                        .decode_to_string(&bytes[offset..], &mut output, !stream);
                offset += read;
                if result == encoding_rs::CoderResult::InputEmpty {
                    break;
                }
            }
        }
        if !stream || invalid {
            self.decoder = Self::decoder(self.encoding, self.ignore_bom);
        }
        if invalid {
            Err(Exception::throw_type(
                &ctx,
                "The encoded data was not valid",
            ))
        } else {
            Ok(output)
        }
    }
}
