use std::rc::Rc;

use gloo::events::EventListener;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlButtonElement, HtmlDivElement, HtmlImageElement, HtmlInputElement, HtmlParagraphElement};

use crate::api::{can_view_inventory, collectibles_account_value, exchange_rate, profile_info};

mod utils;
mod api;

trait WrappedGetElementById {
    fn wr_get_element_by_id<T: JsCast>(&self, id: &str) -> T;
}

impl WrappedGetElementById for Document {
    fn wr_get_element_by_id<T: JsCast>(&self, id: &str) -> T {
        self
            .get_element_by_id(id)
            .unwrap()
            .dyn_into::<T>()
            .unwrap()
    }
}


struct Application {
    document: Document,
    account_id_input: HtmlInputElement,
    find_account_value_button: HtmlButtonElement,
    error_text_div: HtmlDivElement,
    loading_bar: HtmlDivElement,
    account_info_div: HtmlDivElement,
    avatar: HtmlImageElement,
    username: HtmlDivElement,
    robux_value: HtmlParagraphElement,
    robux_value_in_euro: HtmlParagraphElement,
}

impl Application {
    fn new() -> Application {
        let document: Document = gloo::utils::document();

        Application {
            document: gloo::utils::document(),
            account_id_input: document.wr_get_element_by_id::<HtmlInputElement>("account_id_input"),
            find_account_value_button: document.wr_get_element_by_id::<HtmlButtonElement>("find_account_value_button"),
            error_text_div: document.wr_get_element_by_id::<HtmlDivElement>("error-text"),
            loading_bar: document.wr_get_element_by_id::<HtmlDivElement>("loading-bar"),
            account_info_div: document.wr_get_element_by_id::<HtmlDivElement>("account-info"),
            avatar: document.wr_get_element_by_id::<HtmlImageElement>("avatar"),
            username: document.wr_get_element_by_id::<HtmlDivElement>("username"),
            robux_value: document.wr_get_element_by_id::<HtmlParagraphElement>("robux-value"),
            robux_value_in_euro: document.wr_get_element_by_id::<HtmlParagraphElement>("robux-value-in-euro"),
        }
    }

    async fn init(&self) {
        let exchange_rate_p = self.document.wr_get_element_by_id::<HtmlParagraphElement>("exchange-rate");
        match exchange_rate().await {
            Ok(exchange_rate) => {
                exchange_rate_p.set_inner_text(&format!("{} Robux per 1€", exchange_rate.robux_per_euro));
            }
            Err(_) => {
                exchange_rate_p.set_inner_text("0 Robux per 1€")
            }
        }

        self.account_id_input.set_disabled(false);
        self.find_account_value_button.set_disabled(false);
    }

    fn set_error(&self, error: &str) {
        self.error_text_div.set_inner_text(error);
        self.error_text_div.set_hidden(false);

        self.account_info_div.class_list().set_value("hidden");
        self.loading_bar.set_hidden(true);

        self.account_id_input.set_disabled(false);
        self.find_account_value_button.set_disabled(false);
    }

    fn lock_ui(&self) {
        self.account_id_input.set_disabled(true);
        self.find_account_value_button.set_disabled(true);
        self.account_info_div.class_list().set_value("hidden");
        self.loading_bar.set_hidden(false);
        self.error_text_div.set_inner_html("");
        self.error_text_div.set_hidden(true);
    }

    fn unlock_ui(&self) {
        self.account_id_input.set_disabled(false);
        self.find_account_value_button.set_disabled(false);
        self.account_info_div.class_list().set_value("");
        self.loading_bar.set_hidden(true);
    }
}

#[wasm_bindgen]
pub async fn run() {
    utils::set_panic_hook(); // Error handling

    let app = Rc::new(Application::new());
    app.init().await;

    let app2 = Rc::clone(&app);
    let app3 = Rc::clone(&app);

    EventListener::new(&app.find_account_value_button, "click", move |_| {
        let inner_app = Rc::clone(&app2);

        let input_value = inner_app.account_id_input.value();

        inner_app.lock_ui();

        spawn_local(async move {
            if !input_value.is_empty() {
                let id = input_value.parse::<u64>().unwrap();

                match can_view_inventory(id).await {
                    Ok(can_view) => if can_view {
                        let value = collectibles_account_value(id).await.unwrap();
                        inner_app.robux_value.set_inner_text(&format!("Robux: {}", value.total_robux));
                        inner_app.robux_value_in_euro.set_inner_text(&format!("Euros: {}€", value.in_euro));
                    } else {
                        inner_app.set_error("Please make sure the account has inventory set as public");
                        return;
                    }
                    Err(_) => {
                        inner_app.set_error("Please make sure the account has inventory set as public");
                        return;
                    }
                }

                match profile_info(id).await {
                    Ok(info) => {
                        inner_app.avatar.set_src(&info.avatar);
                        inner_app.avatar.set_alt(&format!("{}'s avatar", info.avatar));
                        inner_app.username.set_inner_text(&info.username);
                    }
                    Err(_) => {
                        inner_app.set_error("Error getting profile info");
                        return;
                    }
                };
            } else {
                inner_app.set_error("Please insert a valid account id");
                return
            }

            inner_app.unlock_ui()
        });
    }).forget();

    let mut input_checker: Option<u64> = None;

    EventListener::new(&app.account_id_input, "input", move |event| {
        let inner_app = Rc::clone(&app3);

        let input_value = event
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap()
            .value();

        if input_value.is_empty() {
            input_checker = None
        } else {
            match input_value.parse::<u64>() {
                Ok(i) => {
                    input_checker = Some(i);
                }
                Err(_) => match input_checker {
                    Some(i) => inner_app.account_id_input.set_value(&format!("{i}")),
                    None => inner_app.account_id_input.set_value(""),
                },
            }
        }
    }).forget();
}
