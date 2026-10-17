use rquickjs::{JsLifetime, Result, class::Trace};

#[derive(Trace, JsLifetime)]
#[wabou_bindgen::js_class(name = "MarkerFixture")]
#[rquickjs::class]
struct MarkerFixture {
    value: String,
}

#[allow(dead_code)]
#[wabou_bindgen::js_type(fields(value = string))]
struct MarkerDto {
    value: String,
}

#[wabou_bindgen::js_module(name = "marker")]
#[rquickjs::module(rename_vars = "camelCase")]
mod marker_host {
    #[rquickjs::function]
    #[wabou_bindgen::js(args(value = string), returns = string)]
    pub fn echo(value: String) -> String {
        value
    }
}

#[rquickjs::methods]
impl MarkerFixture {
    #[qjs(constructor)]
    #[wabou_bindgen::js(args(value = string))]
    fn new(value: String) -> Result<Self> {
        Ok(Self { value })
    }

    #[qjs(get)]
    #[wabou_bindgen::js(returns = string)]
    fn value(&self) -> String {
        self.value.clone()
    }
}

#[test]
fn source_markers_coexist_with_rquickjs_macros() {
    let fixture = MarkerFixture {
        value: "marked".into(),
    };
    assert_eq!(fixture.value(), "marked");
    assert_eq!(marker_host::echo("host".into()), "host");
}
