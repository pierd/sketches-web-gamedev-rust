use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{window, Element};


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Core {
    content: Element,
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new(content_id: String) -> Self {
        let selector = format!("#{}", content_id);

        let window = window().unwrap();
        let document = window.document().unwrap();
        let element = document
            .query_selector(&selector)
            .expect("query_selector failed")
            .expect(&format!("No element with selector: {}", content_id));
        Self { content: element }
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.content.set_inner_html("<h1>Hello, World!</h1>");
    }
}
