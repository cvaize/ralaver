use actix_web::{ Error, HttpResponse, Result };

pub async fn app() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css")
        .insert_header(("content-encoding", "gzip"))
        .body(RESOURCES_BUILD_APP_CSS_GZ))
}

pub async fn normalize() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_LIBRARIES_NORMALIZE_CSS))
}

pub async fn layout() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_LAYOUT_CSS))
}

pub async fn sidebar() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_SIDEBAR_CSS))
}

pub async fn accordion() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_ACCORDION_CSS))
}

pub async fn breadcrumb() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_BREADCRUMB_CSS))
}

pub async fn tabs() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_TABS_CSS))
}

pub async fn alert() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_ALERT_CSS))
}

pub async fn btn() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_BTN_CSS))
}

pub async fn collapse() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_COLLAPSE_CSS))
}

pub async fn dropdown() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_DROPDOWN_CSS))
}

pub async fn modal() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_MODAL_CSS))
}

pub async fn pagination() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_PAGINATION_CSS))
}

pub async fn table() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_TABLE_CSS))
}

pub async fn b_checkbox() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_B_CHECKBOX_CSS))
}

pub async fn b_radio() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_B_RADIO_CSS))
}

pub async fn b_tabs() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_B_TABS_CSS))
}

pub async fn c_checkbox() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_C_CHECKBOX_CSS))
}

pub async fn checkbox() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_CHECKBOX_CSS))
}

pub async fn color_checkbox() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_COLOR_CHECKBOX_CSS))
}

pub async fn field() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_FIELD_CSS))
}

pub async fn input() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_INPUT_CSS))
}

pub async fn menu() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_MENU_CSS))
}

pub async fn radio() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_RADIO_CSS))
}

pub async fn s_collapse() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_S_COLLAPSE_CSS))
}

pub async fn c_radio() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_C_RADIO_CSS))
}

pub async fn tag() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_TAG_CSS))
}

pub async fn search_group() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_SEARCH_GROUP_CSS))
}

pub async fn dark_mode() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css").body(RESOURCES_COMPONENTS_DARK_MODE_CSS))
}

static RESOURCES_BUILD_APP_CSS_GZ: &'static [u8] = include_bytes!("../../../resources/build/app.min.css.gz");
static RESOURCES_LIBRARIES_NORMALIZE_CSS: &str = include_str!("../../../resources/libraries/normalize/normalize.css");
static RESOURCES_COMPONENTS_LAYOUT_CSS: &str = include_str!("../../../resources/components/layout/layout.css");
static RESOURCES_COMPONENTS_SIDEBAR_CSS: &str = include_str!("../../../resources/components/sidebar/sidebar.css");
static RESOURCES_COMPONENTS_ACCORDION_CSS: &str = include_str!("../../../resources/components/accordion/accordion.css");
static RESOURCES_COMPONENTS_BREADCRUMB_CSS: &str = include_str!("../../../resources/components/breadcrumb/breadcrumb.css");
static RESOURCES_COMPONENTS_TABS_CSS: &str = include_str!("../../../resources/components/tabs/tabs.css");
static RESOURCES_COMPONENTS_ALERT_CSS: &str = include_str!("../../../resources/components/alert/alert.css");
static RESOURCES_COMPONENTS_BTN_CSS: &str = include_str!("../../../resources/components/btn/btn.css");
static RESOURCES_COMPONENTS_COLLAPSE_CSS: &str = include_str!("../../../resources/components/collapse/collapse.css");
static RESOURCES_COMPONENTS_DROPDOWN_CSS: &str = include_str!("../../../resources/components/dropdown/dropdown.css");
static RESOURCES_COMPONENTS_MODAL_CSS: &str = include_str!("../../../resources/components/modal/modal.css");
static RESOURCES_COMPONENTS_PAGINATION_CSS: &str = include_str!("../../../resources/components/pagination/pagination.css");
static RESOURCES_COMPONENTS_TABLE_CSS: &str = include_str!("../../../resources/components/table/table.css");
static RESOURCES_COMPONENTS_B_CHECKBOX_CSS: &str = include_str!("../../../resources/components/b-checkbox/b-checkbox.css");
static RESOURCES_COMPONENTS_B_RADIO_CSS: &str = include_str!("../../../resources/components/b-radio/b-radio.css");
static RESOURCES_COMPONENTS_B_TABS_CSS: &str = include_str!("../../../resources/components/b-tabs/b-tabs.css");
static RESOURCES_COMPONENTS_C_CHECKBOX_CSS: &str = include_str!("../../../resources/components/c-checkbox/c-checkbox.css");
static RESOURCES_COMPONENTS_CHECKBOX_CSS: &str = include_str!("../../../resources/components/checkbox/checkbox.css");
static RESOURCES_COMPONENTS_COLOR_CHECKBOX_CSS: &str = include_str!("../../../resources/components/color-checkbox/color-checkbox.css");
static RESOURCES_COMPONENTS_FIELD_CSS: &str = include_str!("../../../resources/components/field/field.css");
static RESOURCES_COMPONENTS_INPUT_CSS: &str = include_str!("../../../resources/components/input/input.css");
static RESOURCES_COMPONENTS_MENU_CSS: &str = include_str!("../../../resources/components/menu/menu.css");
static RESOURCES_COMPONENTS_RADIO_CSS: &str = include_str!("../../../resources/components/radio/radio.css");
static RESOURCES_COMPONENTS_S_COLLAPSE_CSS: &str = include_str!("../../../resources/components/s-collapse/s-collapse.css");
static RESOURCES_COMPONENTS_C_RADIO_CSS: &str = include_str!("../../../resources/components/c-radio/c-radio.css");
static RESOURCES_COMPONENTS_TAG_CSS: &str = include_str!("../../../resources/components/tag/tag.css");
static RESOURCES_COMPONENTS_SEARCH_GROUP_CSS: &str = include_str!("../../../resources/components/search-group/search-group.css");
static RESOURCES_COMPONENTS_DARK_MODE_CSS: &str = include_str!("../../../resources/components/dark-mode/dark-mode.css");