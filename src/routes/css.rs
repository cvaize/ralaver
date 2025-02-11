use actix_web::{web};
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/css/app.css").route(web::get().to(controllers::css::app)));
    cfg.service(web::resource("/css/libraries/normalize/normalize.css").route(web::get().to(controllers::css::normalize)));
    cfg.service(web::resource("/css/components/layout/layout.css").route(web::get().to(controllers::css::layout)));
    cfg.service(web::resource("/css/components/sidebar/sidebar.css").route(web::get().to(controllers::css::sidebar)));
    cfg.service(web::resource("/css/components/accordion/accordion.css").route(web::get().to(controllers::css::accordion)));
    cfg.service(web::resource("/css/components/breadcrumb/breadcrumb.css").route(web::get().to(controllers::css::breadcrumb)));
    cfg.service(web::resource("/css/components/tabs/tabs.css").route(web::get().to(controllers::css::tabs)));
    cfg.service(web::resource("/css/components/alert/alert.css").route(web::get().to(controllers::css::alert)));
    cfg.service(web::resource("/css/components/btn/btn.css").route(web::get().to(controllers::css::btn)));
    cfg.service(web::resource("/css/components/collapse/collapse.css").route(web::get().to(controllers::css::collapse)));
    cfg.service(web::resource("/css/components/dropdown/dropdown.css").route(web::get().to(controllers::css::dropdown)));
    cfg.service(web::resource("/css/components/modal/modal.css").route(web::get().to(controllers::css::modal)));
    cfg.service(web::resource("/css/components/pagination/pagination.css").route(web::get().to(controllers::css::pagination)));
    cfg.service(web::resource("/css/components/table/table.css").route(web::get().to(controllers::css::table)));
    cfg.service(web::resource("/css/components/b-checkbox/b-checkbox.css").route(web::get().to(controllers::css::b_checkbox)));
    cfg.service(web::resource("/css/components/b-radio/b-radio.css").route(web::get().to(controllers::css::b_radio)));
    cfg.service(web::resource("/css/components/b-tabs/b-tabs.css").route(web::get().to(controllers::css::b_tabs)));
    cfg.service(web::resource("/css/components/c-checkbox/c-checkbox.css").route(web::get().to(controllers::css::c_checkbox)));
    cfg.service(web::resource("/css/components/checkbox/checkbox.css").route(web::get().to(controllers::css::checkbox)));
    cfg.service(web::resource("/css/components/color-checkbox/color-checkbox.css").route(web::get().to(controllers::css::color_checkbox)));
    cfg.service(web::resource("/css/components/field/field.css").route(web::get().to(controllers::css::field)));
    cfg.service(web::resource("/css/components/input/input.css").route(web::get().to(controllers::css::input)));
    cfg.service(web::resource("/css/components/menu/menu.css").route(web::get().to(controllers::css::menu)));
    cfg.service(web::resource("/css/components/radio/radio.css").route(web::get().to(controllers::css::radio)));
    cfg.service(web::resource("/css/components/s-collapse/s-collapse.css").route(web::get().to(controllers::css::s_collapse)));
    cfg.service(web::resource("/css/components/c-radio/c-radio.css").route(web::get().to(controllers::css::c_radio)));
    cfg.service(web::resource("/css/components/tag/tag.css").route(web::get().to(controllers::css::tag)));
    cfg.service(web::resource("/css/components/search-group/search-group.css").route(web::get().to(controllers::css::search_group)));
    cfg.service(web::resource("/css/components/dark-mode/dark-mode.css").route(web::get().to(controllers::css::dark_mode)));
}