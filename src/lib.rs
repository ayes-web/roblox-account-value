use std::rc::Rc;

use gloo::events::EventListener;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Document, HtmlAnchorElement, HtmlButtonElement, HtmlDivElement, HtmlImageElement,
    HtmlInputElement, HtmlParagraphElement, HtmlSpanElement,
};

use crate::api::{can_view_inventory, collectibles_account_value, exchange_rate, profile_info};

mod api;
mod utils;

trait WrappedGetElementById {
    fn wr_get_element_by_id<T: JsCast>(&self, id: &str) -> T;
}

impl WrappedGetElementById for Document {
    fn wr_get_element_by_id<T: JsCast>(&self, id: &str) -> T {
        self.get_element_by_id(id).unwrap().dyn_into::<T>().unwrap()
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
    username: HtmlSpanElement,
    displayname: HtmlSpanElement,
    username_holder: HtmlAnchorElement,
    robux_value: HtmlDivElement,
    robux_value_in_euro: HtmlDivElement,
    inventory: HtmlDivElement,
}

impl Application {
    fn new() -> Application {
        let document: Document = gloo::utils::document();

        Application {
            document: gloo::utils::document(),
            account_id_input: document.wr_get_element_by_id("account_id_input"),
            find_account_value_button: document.wr_get_element_by_id("find_account_value_button"),
            error_text_div: document.wr_get_element_by_id("error-text"),
            loading_bar: document.wr_get_element_by_id("loading-bar"),
            account_info_div: document.wr_get_element_by_id("account-info"),
            avatar: document.wr_get_element_by_id("avatar"),
            username: document.wr_get_element_by_id("username"),
            displayname: document.wr_get_element_by_id("displayname"),
            username_holder: document.wr_get_element_by_id("username-holder"),
            robux_value: document.wr_get_element_by_id("robux-value"),
            robux_value_in_euro: document.wr_get_element_by_id("robux-value-in-euro"),
            inventory: document.wr_get_element_by_id("inventory"),
        }
    }

    async fn init(&self) {
        let exchange_rate_p: HtmlParagraphElement =
            self.document.wr_get_element_by_id("exchange-rate");
        match exchange_rate().await {
            Ok(exchange_rate) => {
                exchange_rate_p.set_inner_text(&format!(
                    "{} Robux per 1€",
                    exchange_rate.robux_per_euro
                ));
            }
            Err(_) => exchange_rate_p.set_inner_text("0 Robux per 1€"),
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
            if input_value.is_empty() {
                inner_app.set_error("Please insert a valid account id");
                return;
            }

            // Should be a valid number due to input checker
            let id = input_value.parse::<u64>().unwrap();

            if matches!(can_view_inventory(id).await, Err(_) | Ok(false)) {
                inner_app.set_error("Please make sure the account has inventory set as public");
                return;
            }

            let account_value = collectibles_account_value(id).await.unwrap();
            inner_app
                .robux_value
                .set_inner_text(&format!("Robux: {}", account_value.total_robux));
            inner_app
                .robux_value_in_euro
                .set_inner_text(&format!("Euros: {}€", account_value.in_euro));



            if account_value.collectibles.is_empty() {
                inner_app.inventory.set_inner_html("No items found :(");
            } else {
                inner_app.inventory.set_inner_html(
                    &account_value
                .collectibles
                .iter()
                .map(|collectible| {
                    let limited_row = match collectible.serialnumber {
                        Some(serialnumber) => format!("<div class='collectible-serialnumber'>#{serialnumber}</div>"),
                        None => String::new(),
                    };

                    let html = format!(
                        "<div class='collectible'>
                            <div class='collectible-title'>
                                <a href='https://www.roblox.com/catalog/{id}' target='_blank'>{name}</a>
                            </div>

                            <div class='collectible-thumbnail'>
                                <img class='no-select' alt='{name}' src='{thumbnail}'>
                                {limited_row}
                            </div>

                            <div class='collectible-robux'>
                                {price} robux
                            </div>
                        </div>",
                        thumbnail = collectible.thumbnail,
                        id = collectible.id,
                        name = collectible.name,
                        price = collectible.price
                    );

                    html
                })
                .collect::<String>()
                );
            }


            match profile_info(id).await {
                Ok(info) => {
                    inner_app.avatar.set_src(&info.avatar);
                    inner_app
                        .avatar
                        .set_alt(&format!("{}'s avatar", info.avatar));

                    inner_app
                        .username_holder
                        .set_href(&format!("https://www.roblox.com/users/{id}/profile"));
                    inner_app.displayname.set_inner_text(&info.displayname);
                    inner_app
                        .username
                        .set_inner_text(&format!("@{}", info.username));
                }
                Err(_) => {
                    inner_app.set_error("Error getting profile info");
                    return;
                }
            };

            inner_app.unlock_ui()
        });
    })
    .forget();

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
    })
    .forget();
}
