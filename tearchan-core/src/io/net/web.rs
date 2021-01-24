use js_sys::{ArrayBuffer, Promise, Uint8Array};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, Event, FileReader, ProgressEvent, Request, RequestInit, RequestMode, Response,
};

pub async fn get_request_as_binaries(url: &str) -> Result<Vec<u8>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::SameOrigin);

    let request = Request::new_with_str_and_init(url, &opts)?;
    let _ = request.headers().set("Accept", "application/*");

    let window = web_sys::window().unwrap_throw();
    let response_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response_value.dyn_into().unwrap_throw();
    let (sender, receiver) = std::sync::mpsc::channel();
    let blob: Blob = JsFuture::from(response.blob()?)
        .await?
        .dyn_into()
        .unwrap_throw();

    JsFuture::from(Promise::new(&mut |resolve, reject| {
        let reader = FileReader::new().unwrap_throw();
        let state = Rc::new(RefCell::new(None));
        let sender = sender.clone();

        let onload = {
            let state = state.clone();
            let reader = reader.clone();

            Closure::once(move |_: ProgressEvent| {
                *state.borrow_mut() = None;

                let data: ArrayBuffer = reader.result().unwrap().unchecked_into();
                let data: Vec<u8> = Uint8Array::new(&data).to_vec();

                sender.send(data).unwrap();
                resolve.call0(&JsValue::NULL).unwrap();
            })
        };

        let onerror = {
            let state = state.clone();
            // let reader = reader.clone();

            Closure::once(move |_: Event| {
                *state.borrow_mut() = None;

                // let error = reader.error().unwrap_throw();
                reject.call0(&JsValue::NULL).unwrap_throw();
            })
        };

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        *state.borrow_mut() = Some((onload, onerror));

        reader.read_as_array_buffer(&blob).unwrap_throw();
    }))
    .await?;

    Ok(receiver.try_recv().unwrap_throw())
}
