//! UI assembly: builds the AppShell once and the People page (form + table)
//! once. Handlers consume these builders to produce render messages.

use std::collections::HashMap;

use marionette::builders::{
    AppShell, Button, Container, DataTable, Form, Heading, NavItem, Select, SelectOption, SideNav,
    SurfaceMount, TableColumn, Text, TextInput,
};
use marionette_protocol::{Component, ComponentAction};

use crate::handlers;

/// Country options for the form's Select. Two-letter ISO codes — same set
/// echoed back as the row's `country` cell.
#[must_use]
pub fn country_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "ch".into(),
            label: "Switzerland".into(),
        },
        SelectOption {
            value: "de".into(),
            label: "Germany".into(),
        },
        SelectOption {
            value: "fr".into(),
            label: "France".into(),
        },
        SelectOption {
            value: "us".into(),
            label: "United States".into(),
        },
        SelectOption {
            value: "jp".into(),
            label: "Japan".into(),
        },
    ]
}

/// Build the AppShell tree. Returns the flat node list and the slot ids the
/// caller needs to compose into a `RenderMessage`.
#[must_use]
pub fn build_app_shell() -> Vec<(String, Component)> {
    // ---- Sidebar: one nav entry to the People page. ----
    let nav_people = {
        let mut action = ComponentAction::click("navigate");
        action
            .extra
            .insert("payload".into(), serde_json::json!({ "target": "people" }));
        NavItem::new("People", "/people")
            .id("nav-people")
            .action(action)
            .build()
    };
    let (sidebar_root, sidebar_desc) = SideNav::new()
        .id("shell-side-nav")
        .children(vec![nav_people])
        .build_tree();

    // ---- Header: title only. ----
    let header_title = Heading::new("Tutorial: People App")
        .id("header-title")
        .build();
    let (header_root, header_desc) = Container::new()
        .id("shell-header")
        .children(vec![header_title])
        .build_tree();

    // ---- Footer: tag line + bound connection status. ----
    let footer_tagline = Text::new("Tutorial app · in-memory store")
        .id("footer-tagline")
        .build();
    let footer_status = Text::new("connected")
        .id("footer-connection-status")
        .bind("/system/connectionStatus")
        .build();
    let (footer_root, footer_desc) = Container::new()
        .id("shell-footer")
        .children(vec![footer_tagline, footer_status])
        .build_tree();

    // ---- Main slot: SurfaceMount for the `content` sub-surface. ----
    let content_mount = SurfaceMount::new("content")
        .id("shell-content-mount")
        .build();

    let mut descendants: Vec<(String, Component)> = Vec::new();
    descendants.extend(sidebar_desc);
    descendants.extend(header_desc);
    descendants.extend(footer_desc);

    AppShell::new()
        .id("app-shell-root")
        .sidebar(sidebar_root)
        .header(header_root)
        .footer(footer_root)
        .main(content_mount)
        .with_descendants(descendants)
        .build_with_children()
}

/// Build the People page (form + table) for the `content` sub-surface.
/// Returns `(root_id, nodes_map, initial_data)` ready to drop into
/// `RenderMessage`.
#[must_use]
pub fn build_people_page(
    rows: Vec<crate::state::Person>,
) -> (String, HashMap<String, Component>, serde_json::Value) {
    // ---- Form fields. Each binds into /form/{field}. ----
    let name_field = TextInput::new("Name")
        .id("field-name")
        .placeholder("Ada Lovelace")
        .bind("/form/name")
        .build();
    let email_field = TextInput::new("Email")
        .id("field-email")
        .placeholder("ada@example.com")
        .bind("/form/email")
        .build();
    let country_field = Select::new("Country", country_options())
        .id("field-country")
        .bind("/form/country")
        .build();
    let submit = Button::new("Add person")
        .id("btn-add")
        .action(ComponentAction::submit(handlers::people::APP_ADD_PERSON))
        .build();

    let (form_root, form_desc) = Form::new()
        .id("people-form")
        .bind("/form")
        .children(vec![name_field, email_field, country_field, submit])
        .build_tree();

    // ---- Table. Three columns; rows live at /people. ----
    let columns = vec![
        TableColumn::new("name", "Name").sortable(),
        TableColumn::new("email", "Email"),
        TableColumn::new("country", "Country"),
    ];
    let (table_id, table_component) = DataTable::new(columns)
        .id("people-table")
        .bind("/people")
        .row_id_key("id")
        .build();

    // Wrap form in a card-like container; wrap table heading + table.
    let form_card_heading = Heading::new("Add a person")
        .id("form-card-heading")
        .level(2)
        .build();
    let (form_card_root, form_card_desc) = Container::new()
        .id("form-card")
        .children(vec![form_card_heading, form_root])
        .build_tree();

    let table_card_heading = Heading::new("People")
        .id("table-card-heading")
        .level(2)
        .build();
    let (table_card_root, table_card_desc) = Container::new()
        .id("table-card")
        .children(vec![
            table_card_heading,
            (table_id.clone(), table_component.clone()),
        ])
        .build_tree();

    // ---- Page root: stacks the two cards. ----
    let page_nodes = Container::new()
        .id("people-page")
        .children(vec![form_card_root, table_card_root])
        .build_with_children();

    // Flatten everything into one map.
    let mut nodes: HashMap<String, Component> = HashMap::new();
    for (id, c) in page_nodes {
        nodes.insert(id, c);
    }
    for (id, c) in form_desc {
        nodes.insert(id, c);
    }
    for (id, c) in form_card_desc {
        nodes.insert(id, c);
    }
    for (id, c) in table_card_desc {
        nodes.insert(id, c);
    }

    let data = serde_json::json!({
        "form": { "name": "", "email": "", "country": "" },
        "people": rows,
    });

    ("people-page".to_string(), nodes, data)
}
