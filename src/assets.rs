use collect_mac::*;
use futures::*;
use js_sys::*;
use log::*;
use std::cell::RefCell;
use std::collections::*;
use std::mem;
use std::ops::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::*;

/// Stores assets that have been loaded. Currently, a URL can be loaded as a `Vec<u8>` or
/// an `HtmlImageElement`.
pub struct Assets {
    assets: HashMap<String, Vec<u8>>,
    images: HashMap<String, HtmlImageElement>,
}

impl Assets {
    /// Loads one or more assets from URLs. This function loads the URLs asynchronously, and
    /// takes a callback to be called once all the assets are loaded.
    ///
    /// This can also load images, as `HtmlImageElement`s. It's also possible to load images
    /// as regular files using the `image` crate.
    ///
    /// This loads all assets concurrently. It's intended for large assets; small assets should
    /// usually be loaded at compile time with `include_str!` or `include_bytes!`.
    ///
    /// Panics if any asset can't be loaded.
    pub fn load(asset_urls: Vec<String>, image_urls: Vec<String>, callback: Box<dyn Fn(Assets)>) {
        assert!(!asset_urls.is_empty() || !image_urls.is_empty());

        let loaded_assets: Rc<RefCell<HashMap<String, Vec<u8>>>> =
            Rc::new(RefCell::new(collect![]));
        let loaded_images: Rc<RefCell<HashMap<String, HtmlImageElement>>> =
            Rc::new(RefCell::new(collect![]));

        let loaded_assets2 = loaded_assets.clone();
        let loaded_images2 = loaded_images.clone();
        let num_assets = asset_urls.len();
        let num_images = image_urls.len();
        let check_if_done_loading = Rc::new(move || {
            if loaded_assets2.borrow().len() == num_assets
                && loaded_images2.borrow().len() == num_images
            {
                callback(Assets {
                    assets: mem::replace(&mut loaded_assets2.borrow_mut(), collect![]),
                    images: mem::replace(&mut loaded_images2.borrow_mut(), collect![]),
                });
            }
        });

        for asset_url in asset_urls {
            let asset_url2 = asset_url.clone();
            let loaded_assets = loaded_assets.clone();
            let check_if_done_loading = check_if_done_loading.clone();

            let mut request_init = RequestInit::new();
            request_init.method("GET");
            request_init.mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&asset_url, &request_init).unwrap();
            let request_promise = window().unwrap().fetch_with_request(&request);

            let future = JsFuture::from(request_promise)
                .and_then(move |response| {
                    let response: Response = response.dyn_into().unwrap();
                    if !response.ok() {
                        panic!("Unable to load asset: {:?}", asset_url2);
                    }
                    response.array_buffer()
                })
                .and_then(JsFuture::from)
                .and_then(move |array_buffer| {
                    let array_buffer: ArrayBuffer = array_buffer.into();
                    let array: Uint8Array = Uint8Array::new(&array_buffer);
                    let mut dst = vec![0; array_buffer.byte_length() as usize];
                    array.copy_to(&mut dst);
                    loaded_assets.borrow_mut().insert(asset_url.clone(), dst);
                    check_if_done_loading();
                    future::ok(JsValue::NULL)
                });
            future_to_promise(future);
        }

        for image_url in image_urls {
            let loaded_images = loaded_images.clone();
            let check_if_done_loading = check_if_done_loading.clone();

            let image_element = window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("img")
                .unwrap()
                .dyn_into::<HtmlImageElement>()
                .unwrap();
            let image_url2 = image_url.clone();
            let image_url3 = image_url.clone();
            let image_element2 = image_element.clone();

            let onload_handler = Rc::new(RefCell::new(None));
            let onload_handler2 = onload_handler.clone();
            *onload_handler.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                loaded_images.borrow_mut().insert(image_url2.clone(), image_element2.clone());
                debug!("Loaded {}", image_url2);
                check_if_done_loading();
                onload_handler2.borrow_mut().take();
            }) as Box<dyn FnMut()>));
            image_element.set_onload(Some(
                onload_handler.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            ));

            let onerror_handler = Rc::new(RefCell::new(None));
            let onerror_handler2 = onerror_handler.clone();
            *onerror_handler.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                onerror_handler2.borrow_mut().take();
                panic!("Unable to load image: {:?}", image_url3);
            }) as Box<dyn FnMut()>));
            image_element.set_onerror(Some(
                onerror_handler.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            ));

            image_element.set_src(&image_url);
        }
    }

    /// Returns a reference to the given asset.
    pub fn get(&self, url: &str) -> Option<&[u8]> {
        self.assets.get(url).map(|x| x.as_slice())
    }

    /// Removes the given asset and returns it.
    pub fn remove(&mut self, url: &str) -> Option<Vec<u8>> {
        self.assets.remove(url)
    }

    /// Returns the given image.
    pub fn get_image(&self, url: &str) -> Option<&HtmlImageElement> {
        self.images.get(url)
    }
}
