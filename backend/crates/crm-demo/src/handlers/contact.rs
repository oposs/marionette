use std::collections::{HashMap, HashSet};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, ColumnKind, Container, DataTable, FieldSeparator, FieldSet, Filter, Form, Heading,
    Select, SelectOption, Switch, TableColumn, Text, Textarea, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::{company, contact, contact_tag, interaction, listmonk_sync, note, tag, user};

/// Format current UTC time as SQLite datetime string.
fn now_sqlite() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Date-range filter payload shape produced by the Phase 13 DataTable
/// `date-range` filter kind. Sent as `{from, to}` inside the filter values
/// object (see D-C3).
#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct DateRange {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

/// Filter payload for the contact list screen.
///
/// Matches the new Phase 13 filter shape (D-B2 / D-C3): a flat map keyed by
/// filter id where each value matches the filter kind. `search` and
/// `tag_filter_text` are text filters; `company_filter` is a select whose
/// value comes through as a string (parsed to i32 in the handler);
/// `date` is a date-range filter with nested `{from, to}` strings.
#[derive(Debug, Deserialize, Default)]
pub struct ContactFilterParams {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub company_filter: Option<String>,
    #[serde(default)]
    pub tag_filter_text: Option<String>,
    #[serde(default)]
    pub date: Option<DateRange>,
}

/// Assign a color from a fixed palette based on tag name hash.
fn tag_color(name: &str) -> &'static str {
    const PALETTE: &[&str] = &[
        "blue", "green", "red", "yellow", "indigo", "purple", "pink", "teal",
    ];
    let hash = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
    PALETTE[(hash as usize) % PALETTE.len()]
}

/// Payload for identifying a contact by ID.
#[derive(Deserialize)]
struct ContactIdPayload {
    contact_id: i32,
}

/// Inner form data for a contact (create or update).
///
/// Phase 14 Plan 08: `notes`, `opt_in`, and `country` are accepted by the
/// handler but not yet persisted — the contact entity doesn't have
/// columns for them. Phase 15 will add the schema. The `#[serde(default)]`
/// + `rename` annotations keep payload-shape tolerance without forcing
/// the frontend to send every field on every submit.
#[derive(Deserialize)]
struct ContactFormData {
    id: Option<i32>,
    name: String,
    email: String,
    phone: Option<String>,
    title: Option<String>,
    company: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    country: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    notes: Option<String>,
    #[serde(default, rename = "optIn")]
    #[allow(dead_code)]
    opt_in: Option<bool>,
}

/// Payload wrapper: the frontend sends all surface data, with form fields
/// nested under the form's bind prefix (e.g. `contactForm`).
#[derive(Deserialize)]
struct ContactSavePayload {
    #[serde(rename = "contactForm")]
    contact_form: ContactFormData,
}

/// Shared helper: build a rendered contact list from the database.
/// Accepts optional search/filter parameters from the action payload.
async fn render_contact_list(ctx: &HandlerContext) -> ActionResult {
    let db = Db::from_context(ctx)?;

    // Extract optional search/filter parameters
    let params: ContactFilterParams = ctx
        .action
        .payload
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Parse the (string) select filter value to i32. Invalid strings are
    // silently dropped (they never reach the SQL layer).
    let company_filter_int: Option<i32> = params
        .company_filter
        .as_deref()
        .and_then(|s| s.parse::<i32>().ok());

    // Build dynamic filter condition
    let mut condition = Condition::all();

    let search_term = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    if let Some(ref q) = search_term {
        condition = condition.add(
            Condition::any()
                .add(contact::Column::ContactName.contains(q.as_str()))
                .add(contact::Column::ContactEmail.contains(q.as_str())),
        );
    }

    if let Some(company_id) = company_filter_int {
        condition = condition.add(contact::Column::ContactCompany.eq(company_id));
    }

    if let Some(ref dr) = params.date {
        if let Some(ref from_date) = dr.from {
            let trimmed = from_date.trim();
            if !trimmed.is_empty() {
                condition =
                    condition.add(contact::Column::ContactCreatedAt.gte(trimmed.to_owned()));
            }
        }
        if let Some(ref to_date) = dr.to {
            let trimmed = to_date.trim();
            if !trimmed.is_empty() {
                // Append time so that the whole day is included
                let to_end = if trimmed.len() == 10 {
                    format!("{trimmed} 23:59:59")
                } else {
                    trimmed.to_owned()
                };
                condition = condition.add(contact::Column::ContactCreatedAt.lte(to_end));
            }
        }
    }

    // Tag filter: parse comma-separated tag names, look up IDs, filter contacts
    let tag_filter_names: Vec<String> = params
        .tag_filter_text
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if !tag_filter_names.is_empty() {
        // Look up tag IDs by name
        let all_tags = tag::Entity::find()
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let matching_tag_ids: Vec<i32> = all_tags
            .iter()
            .filter(|t| tag_filter_names.contains(&t.tag_name.to_lowercase()))
            .map(|t| t.tag_id)
            .collect();

        if matching_tag_ids.is_empty() {
            // No tags match -> no contacts can match
            condition = condition.add(contact::Column::ContactId.eq(-1));
        } else {
            let tagged_contacts = contact_tag::Entity::find()
                .filter(contact_tag::Column::ContactTagTag.is_in(matching_tag_ids))
                .all(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;
            let tagged_ids: HashSet<i32> = tagged_contacts
                .iter()
                .map(|ct| ct.contact_tag_contact)
                .collect();
            if tagged_ids.is_empty() {
                condition = condition.add(contact::Column::ContactId.eq(-1));
            } else {
                condition = condition.add(
                    contact::Column::ContactId.is_in(tagged_ids.into_iter().collect::<Vec<_>>()),
                );
            }
        }
    }

    // Compute total_rows with the SAME WHERE clauses as the page query
    // (D-H2). Clone the composed Condition before the page query consumes it.
    let count_condition = condition.clone();
    let contact_count: u64 = contact::Entity::find()
        .filter(count_condition)
        .count(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // D-H1 / D-H2: the initial render is a single page (page_size rows).
    // The remaining rows stream in via the fetch_rows handler as the user
    // scrolls the DataTable. Prior to Phase 13 this query fetched ALL
    // rows via `.all()`, which masked the infinite-scroll plumbing and
    // made it impossible for the sentinel to fire (rows.length already
    // equalled total_rows on first render — see 13-07 E2E test discovery).
    const INITIAL_PAGE_SIZE: u64 = 50;
    let contacts = contact::Entity::find()
        .find_also_related(company::Entity)
        .filter(condition)
        .order_by_asc(contact::Column::ContactName)
        .offset(0u64)
        .limit(INITIAL_PAGE_SIZE)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // NOTE: the pre-Phase-13 post-filter that re-queried ALL contacts to
    // pick up company-name matches via in-memory filtering is removed
    // because it is incompatible with pagination. The SQL LIKE on
    // ContactName + ContactEmail remains as the authoritative search; a
    // future plan can add a JOIN-based SQL company-name filter if
    // needed. The fetch_rows path is unaffected (it runs its own page
    // query).
    let _ = &search_term;

    // Load tags for all displayed contacts in one batch query
    let contact_ids: Vec<i32> = contacts.iter().map(|(c, _)| c.contact_id).collect();
    let contact_tags_map = if contact_ids.is_empty() {
        HashMap::new()
    } else {
        let ct_rows = contact_tag::Entity::find()
            .filter(contact_tag::Column::ContactTagContact.is_in(contact_ids.clone()))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let all_tags = tag::Entity::find()
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let tag_name_map: HashMap<i32, String> =
            all_tags.into_iter().map(|t| (t.tag_id, t.tag_name)).collect();
        let mut map: HashMap<i32, Vec<String>> = HashMap::new();
        for ct in ct_rows {
            if let Some(name) = tag_name_map.get(&ct.contact_tag_tag) {
                map.entry(ct.contact_tag_contact)
                    .or_default()
                    .push(name.clone());
            }
        }
        map
    };

    // Batch-load Listmonk sync statuses for all displayed contacts
    let sync_statuses: HashMap<i32, (String, Option<String>)> = if contact_ids.is_empty() {
        HashMap::new()
    } else {
        let syncs = listmonk_sync::Entity::find()
            .filter(listmonk_sync::Column::ListmonkSyncContact.is_in(contact_ids.clone()))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        syncs
            .into_iter()
            .map(|s| {
                (
                    s.listmonk_sync_contact,
                    (s.listmonk_sync_status, s.listmonk_sync_error),
                )
            })
            .collect()
    };

    // Load companies for filter dropdown
    let companies = company::Entity::find()
        .order_by_asc(company::Column::CompanyName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let mut company_options = vec![SelectOption {
        value: String::new(),
        label: "All Companies".into(),
    }];
    for co in &companies {
        company_options.push(SelectOption {
            value: co.company_id.to_string(),
            label: co.company_name.clone(),
        });
    }

    // Build UI components
    let heading = Heading::new("Contact Management")
        .id("contact-heading")
        .build();

    let new_button = Button::new("New Contact")
        .id("btn-new-contact")
        .action(ComponentAction::click("contact_new"))
        .build();

    let sync_all_button = Button::new("Sync All to Listmonk")
        .id("btn-sync-all")
        .action(ComponentAction::click("listmonk_sync_all"))
        .build();

    let table = DataTable::new(vec![
        TableColumn::new("name", "Name").sortable(),
        TableColumn::new("email", "Email").sortable(),
        TableColumn::new("phone", "Phone").sortable(),
        TableColumn::new("company", "Company").sortable(),
        TableColumn::new("tags", "Tags"),
        TableColumn::new("sync_status", "Sync"),
        TableColumn::new("created", "Created")
            .sortable()
            .kind(ColumnKind::Date),
        TableColumn::new("actions", "").kind(ColumnKind::Actions),
    ])
    .filter(
        Filter::text("search")
            .label("Search")
            .placeholder("Filter contacts..."),
    )
    .filter(Filter::select("company_filter", company_options).label("Company"))
    .filter(
        Filter::text("tag_filter_text")
            .label("Tag")
            .placeholder("e.g. vip"),
    )
    .filter(Filter::date_range("date").label("Created date"))
    .total_rows(contact_count)
    .source("contact_list")
    .row_id_key("id")
    .page_size(50u32)
    .id("contact-table")
    .bind("/contacts")
    .build();

    let all_children = vec![heading, new_button, sync_all_button, table];

    let container_nodes = Container::new()
        .id("contact-list-root")
        .children(all_children)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    // Build row data with joined company name, tags, and per-row actions
    let rows: Vec<serde_json::Value> = contacts
        .iter()
        .map(|(c, co)| {
            let tags_str = contact_tags_map
                .get(&c.contact_id)
                .map(|names| names.join(", "))
                .unwrap_or_default();
            let sync_label = match sync_statuses.get(&c.contact_id) {
                Some((status, _)) if status == "success" => "Synced",
                Some((_, Some(err))) => err.as_str(),
                Some((_, None)) => "Error",
                None => "Not synced",
            };
            serde_json::json!({
                "id": c.contact_id,
                "name": c.contact_name,
                "email": c.contact_email,
                "phone": c.contact_phone.as_deref().unwrap_or("-"),
                "company": co.as_ref().map(|comp| comp.company_name.as_str()).unwrap_or("-"),
                "tags": tags_str,
                "sync_status": sync_label,
                "created": c.contact_created_at,
                "actions": [
                    { "label": "Edit", "action": { "type": "click", "name": "contact_edit", "payload": { "contact_id": c.contact_id } } },
                    { "label": "Delete", "action": { "type": "click", "name": "contact_delete", "payload": { "contact_id": c.contact_id } } }
                ]
            })
        })
        .collect();

    // Filter state is owned locally by the frontend DataTable component per
    // D-C4; the backend no longer pre-populates initial filter values.
    let data = serde_json::json!({
        "contacts": rows,
    });

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root: "contact-list-root".into(),
            nodes,
            data,
        }),
        nav_active_patch("contacts"),
    ])
}

/// Handle the `contact_list` action: render a DataTable of all contacts.
pub async fn handle_contact_list(ctx: HandlerContext) -> ActionResult {
    render_contact_list(&ctx).await
}

/// Handle the `contact_new` / `contact_edit` action: render a create/edit form.
pub async fn handle_contact_form(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Try to extract contact_id from payload (edit mode) -- if missing, it's create mode
    let contact_id = Payload::<ContactIdPayload>::from_context(&ctx)
        .ok()
        .map(|p| p.0.contact_id);

    let (form_data, form_title) = if let Some(cid) = contact_id {
        let found = contact::Entity::find_by_id(cid)
            .one(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?
            .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;
        (
            // Phase 14 Plan 08: `notes` and `optIn` are surfaced for the
            // new Textarea + Switch primitives. The contact entity does
            // not yet have these columns (Phase 15 will add them), so
            // they seed to empty/false on both new and edit paths and
            // the handler below reads-but-skips-persistence for them.
            serde_json::json!({
                "contactForm": {
                    "id": found.contact_id,
                    "name": found.contact_name,
                    "email": found.contact_email,
                    "phone": found.contact_phone.as_deref().unwrap_or(""),
                    "title": found.contact_title.as_deref().unwrap_or(""),
                    "company": found.contact_company.map(|id| id.to_string()).unwrap_or_default(),
                    "country": "",
                    "notes": "",
                    "optIn": false
                }
            }),
            "Edit Contact",
        )
    } else {
        (
            serde_json::json!({
                "contactForm": {
                    "id": null,
                    "name": "",
                    "email": "",
                    "phone": "",
                    "title": "",
                    "company": "",
                    "country": "",
                    "notes": "",
                    "optIn": false
                }
            }),
            "New Contact",
        )
    };

    // Build company select dropdown options
    let companies = company::Entity::find()
        .order_by_asc(company::Column::CompanyName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let mut options = vec![SelectOption {
        value: String::new(),
        label: "No Company".into(),
    }];
    for co in &companies {
        options.push(SelectOption {
            value: co.company_id.to_string(),
            label: co.company_name.clone(),
        });
    }

    let heading = Heading::new(form_title)
        .id("contact-form-heading")
        .build();

    let back_button = Button::new("← Back")
        .id("contact-form-back")
        .variant("outline")
        .action(ComponentAction::click("contact_list"))
        .build();

    // -- FieldSet 1: Contact information (auto-responsive 2-col grid) --

    let name_input = TextInput::new("Name")
        .id("contact-form-name")
        .bind("/contactForm/name")
        .required(true)
        .build();

    let email_input = TextInput::new("Email")
        .id("contact-form-email")
        .bind("/contactForm/email")
        .input_type("email")
        .description("We will never share your email.")
        .build();

    let phone_input = TextInput::new("Phone")
        .id("contact-form-phone")
        .bind("/contactForm/phone")
        .input_type("tel")
        .build();

    let title_input = TextInput::new("Title")
        .id("contact-form-title")
        .bind("/contactForm/title")
        .build();

    let (contact_info_set, contact_info_descendants) = FieldSet::new()
        .id("contact-info-set")
        .legend("Contact information")
        .children(vec![name_input, email_input, phone_input, title_input])
        .build_tree();

    let separator_1 = FieldSeparator::new()
        .id("contact-form-separator-1")
        .build();

    // -- FieldSet 2: Organisation (company + country-select node-patch demo) --

    let company_select = Select::new("Company", options)
        .id("contact-form-company")
        .bind("/contactForm/company")
        .build();

    // Country select — Phase 12 node-patch demo (D-A6 focus preservation).
    // Changing the country dispatches `contact_country_change`, whose handler
    // emits a PatchMessage that swaps country-specific fields in place via
    // set-node + insert-child + delete-node + remove-child ops on the
    // `content` surface, and additionally insert-child + set-node ops on
    // the `toasts` sub-surface (D-B15 toast lifecycle demo).
    //
    // Phase 14 D-A2: the country-specific swap target is now the
    // `organisation-set` FieldSet (not the outer `contact-form`), because
    // the field is composed inside the Organisation fieldset.
    let country_select = Select::new(
        "Country",
        vec![
            SelectOption {
                value: String::new(),
                label: "Select...".into(),
            },
            SelectOption {
                value: "CH".into(),
                label: "Switzerland".into(),
            },
            SelectOption {
                value: "US".into(),
                label: "United States".into(),
            },
            SelectOption {
                value: "DE".into(),
                label: "Germany".into(),
            },
        ],
    )
    .id("contact-form-country")
    .bind("/contactForm/country")
    .action(ComponentAction::change("contact_country_change"))
    .build();

    let (organisation_set, organisation_descendants) = FieldSet::new()
        .id("organisation-set")
        .legend("Organisation")
        .children(vec![company_select, country_select])
        .build_tree();

    let separator_2 = FieldSeparator::new()
        .id("contact-form-separator-2")
        .build();

    // -- FieldSet 3: Notes + Preferences (exercises Textarea + Switch) --

    let notes_textarea = Textarea::new("Notes")
        .id("contact-form-notes")
        .bind("/contactForm/notes")
        .placeholder("Add notes about this contact...")
        .rows(4u32)
        .full_width(true)
        .build();

    let opt_in_switch = Switch::new("Receive marketing emails")
        .id("contact-form-opt-in")
        .bind("/contactForm/optIn")
        .description("Occasional updates about new features.")
        .build();

    let (preferences_set, preferences_descendants) = FieldSet::new()
        .id("preferences-set")
        .legend("Notes and preferences")
        .children(vec![notes_textarea, opt_in_switch])
        .build_tree();

    // -- Action row (D-D1 Option A: plain Container, flex gap-2 justify-end) --

    let cancel_button = Button::new("Cancel")
        .id("contact-form-cancel")
        .variant("outline")
        .action(ComponentAction::click("contact_list"))
        .build();

    let save_button = Button::new("Save contact")
        .id("contact-form-save")
        .variant("default")
        .action(ComponentAction::submit("contact_save"))
        .build();

    let (action_row, action_row_descendants) = Container::new()
        .id("contact-form-actions")
        .class("flex gap-2 justify-end")
        .children(vec![cancel_button, save_button])
        .build_tree();

    // -- Compose the form --

    let (form_child, form_descendants) = Form::new()
        .id("contact-form")
        .children(vec![
            contact_info_set,
            separator_1,
            organisation_set,
            separator_2,
            preferences_set,
            action_row,
        ])
        .build_tree();

    let mut all_nodes = Vec::new();
    let mut extra_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    all_nodes.push(heading);
    all_nodes.push(back_button);
    all_nodes.push(form_child);
    extra_descendants.extend(contact_info_descendants);
    extra_descendants.extend(organisation_descendants);
    extra_descendants.extend(preferences_descendants);
    extra_descendants.extend(action_row_descendants);
    extra_descendants.extend(form_descendants);

    let mut merged_data = form_data;

    // In edit mode, append tags and notes sections below the form
    if let Some(cid) = contact_id {
        // --- Tags section ---
        let tags_heading = Heading::new("Tags")
            .id("tags-heading")
            .build();
        all_nodes.push(tags_heading);

        // Load current tags for this contact
        let ct_rows = contact_tag::Entity::find()
            .filter(contact_tag::Column::ContactTagContact.eq(cid))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let all_tags = tag::Entity::find()
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let tag_name_map: HashMap<i32, String> =
            all_tags.into_iter().map(|t| (t.tag_id, t.tag_name)).collect();

        for ct in &ct_rows {
            if let Some(name) = tag_name_map.get(&ct.contact_tag_tag) {
                let color = tag_color(name);
                let label = format!("{name} [{color}] [x]");
                let mut action = ComponentAction::click("contact_tag_remove");
                action.extra.insert(
                    "payload".into(),
                    serde_json::json!({ "contact_id": cid, "tag_id": ct.contact_tag_tag }),
                );
                let remove_btn = Button::new(&label)
                    .id(&format!("tag-remove-{}", ct.contact_tag_tag))
                    .action(action)
                    .build();
                all_nodes.push(remove_btn);
            }
        }

        // Add-tag form: text input + submit button wrapped in a Form
        let tag_input = TextInput::new("Add tag...")
            .id("tag-input")
            .bind("/tagForm/name")
            .build();

        let tag_submit = Button::new("Add Tag")
            .id("tag-add")
            .action(ComponentAction::submit("contact_tag_save"))
            .build();

        let (tag_form_child, tag_form_descendants) = Form::new()
            .id("tag-form")
            .children(vec![tag_input, tag_submit])
            .build_tree();
        all_nodes.push(tag_form_child);
        extra_descendants.extend(tag_form_descendants);

        // --- Notes section ---
        let notes = note::Entity::find()
            .filter(note::Column::NoteContact.eq(cid))
            .order_by_desc(note::Column::NoteCreatedAt)
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        // Notes heading
        let notes_heading = Heading::new("Notes")
            .id("notes-heading")
            .build();
        all_nodes.push(notes_heading);

        // Add-note form: text input + submit button wrapped in a Form
        let note_input = TextInput::new("Add a note...")
            .id("note-input")
            .bind("/noteForm/text")
            .build();

        let note_submit = Button::new("Add Note")
            .id("note-submit")
            .action(ComponentAction::submit("note_save"))
            .build();

        let (note_form_child, note_form_descendants) = Form::new()
            .id("note-form")
            .children(vec![note_input, note_submit])
            .build_tree();
        all_nodes.push(note_form_child);
        extra_descendants.extend(note_form_descendants);

        // Render existing notes as Text components
        for n in &notes {
            // Look up author name (N+1 acceptable at demo scale)
            let author_name = user::Entity::find_by_id(n.note_user)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .map(|u| u.user_name)
                .unwrap_or_else(|| "Unknown".into());

            let note_text = format!(
                "[{}] {}: {}",
                n.note_created_at, author_name, n.note_text
            );
            let note_component = Text::new(&note_text)
                .id(&format!("note-{}", n.note_id))
                .build();
            all_nodes.push(note_component);
        }

        // --- Interactions timeline section ---
        let interactions = interaction::Entity::find()
            .filter(interaction::Column::InteractionContact.eq(cid))
            .order_by_desc(interaction::Column::InteractionDate)
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        // Batch-load user names for interaction authors
        let interaction_user_ids: HashSet<i32> =
            interactions.iter().map(|i| i.interaction_user).collect();
        let interaction_users: HashMap<i32, String> = if interaction_user_ids.is_empty() {
            HashMap::new()
        } else {
            let users = user::Entity::find()
                .filter(user::Column::UserId.is_in(interaction_user_ids.into_iter().collect::<Vec<_>>()))
                .all(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;
            users.into_iter().map(|u| (u.user_id, u.user_name)).collect()
        };

        let interactions_heading = Heading::new("Interactions")
            .id("interaction-timeline-heading")
            .build();
        all_nodes.push(interactions_heading);

        // "Log Interaction" button with contact_id payload
        let mut log_action = ComponentAction::click("interaction_form");
        log_action.extra.insert(
            "payload".into(),
            serde_json::json!({ "contact_id": cid }),
        );
        let log_interaction_btn = Button::new("Log Interaction")
            .id("btn-log-interaction")
            .action(log_action)
            .build();
        all_nodes.push(log_interaction_btn);

        // Build interaction timeline as a DataTable
        let timeline_table = DataTable::new(vec![
            TableColumn {
                key: "type_label".into(),
                label: "Type".into(),
                sortable: Some(true),
                ..Default::default()
            },
            TableColumn {
                key: "subject".into(),
                label: "Subject".into(),
                sortable: None,
                ..Default::default()
            },
            TableColumn {
                key: "date".into(),
                label: "Date".into(),
                sortable: Some(true),
                ..Default::default()
            },
            TableColumn {
                key: "logged_by".into(),
                label: "Logged By".into(),
                sortable: None,
                ..Default::default()
            },
            TableColumn {
                key: "notes".into(),
                label: "Notes".into(),
                sortable: None,
                ..Default::default()
            },
        ])
        .id("interaction-timeline")
        .bind("/interactions")
        .build();
        all_nodes.push(timeline_table);

        let interaction_rows: Vec<serde_json::Value> = interactions
            .iter()
            .map(|i| {
                let type_label = match i.interaction_type.as_str() {
                    "call" => "Phone Call",
                    "email" => "Email",
                    "meeting" => "Meeting",
                    _ => &i.interaction_type,
                };
                let user_name = interaction_users
                    .get(&i.interaction_user)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".into());
                serde_json::json!({
                    "type_label": type_label,
                    "subject": i.interaction_subject,
                    "date": i.interaction_date,
                    "logged_by": user_name,
                    "notes": i.interaction_notes.as_deref().unwrap_or("")
                })
            })
            .collect();

        // --- Listmonk Sync section ---
        let sync_record = listmonk_sync::Entity::find()
            .filter(listmonk_sync::Column::ListmonkSyncContact.eq(cid))
            .one(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        let sync_heading = Heading::new("Listmonk Sync")
            .id("sync-heading")
            .build();
        all_nodes.push(sync_heading);

        let status_text = match &sync_record {
            Some(s) if s.listmonk_sync_status == "success" => format!(
                "Synced (subscriber #{}) at {}",
                s.listmonk_sync_subscriber_id.unwrap_or(0),
                s.listmonk_sync_at
            ),
            Some(s) => format!(
                "Error: {} (at {})",
                s.listmonk_sync_error.as_deref().unwrap_or("unknown"),
                s.listmonk_sync_at
            ),
            None => "Not yet synced".to_string(),
        };
        let sync_status = Text::new(&status_text)
            .id("sync-status")
            .build();
        all_nodes.push(sync_status);

        let mut sync_action = ComponentAction::click("listmonk_sync");
        sync_action.extra.insert(
            "payload".into(),
            serde_json::json!({ "contact_id": cid }),
        );
        let sync_button = Button::new("Sync to Listmonk")
            .id("btn-sync")
            .action(sync_action)
            .build();
        all_nodes.push(sync_button);

        // --- Mailing History section ---
        let history_heading = Heading::new("Mailing History")
            .id("history-heading")
            .build();
        all_nodes.push(history_heading);

        let history_data = super::listmonk::get_cached_or_fetch_history(&*db.0, cid).await?;
        let has_history = history_data
            .as_array()
            .is_some_and(|a| !a.is_empty());

        // Refresh button
        let mut refresh_action = ComponentAction::click("listmonk_history_refresh");
        refresh_action.extra.insert(
            "payload".into(),
            serde_json::json!({ "contact_id": cid }),
        );
        let refresh_button = Button::new("Refresh History")
            .id("btn-refresh-history")
            .action(refresh_action)
            .build();
        all_nodes.push(refresh_button);

        if has_history {
            let history_table = DataTable::new(vec![
                TableColumn {
                    key: "campaign".into(),
                    label: "Campaign".into(),
                    sortable: Some(true),
                    ..Default::default()
                },
                TableColumn {
                    key: "date".into(),
                    label: "Date".into(),
                    sortable: Some(true),
                    ..Default::default()
                },
                TableColumn {
                    key: "status".into(),
                    label: "Status".into(),
                    sortable: None,
                    ..Default::default()
                },
            ])
            .id("history-table")
            .bind("/mailingHistory")
            .build();
            all_nodes.push(history_table);
        } else {
            let no_history = Text::new("No mailing history available. Sync contact first, then check back.")
                .id("no-history")
                .build();
            all_nodes.push(no_history);
        }

        // Merge tagForm, noteForm, interactions, and mailingHistory data with contact_id
        if let Some(obj) = merged_data.as_object_mut() {
            obj.insert(
                "tagForm".into(),
                serde_json::json!({ "name": "", "contact_id": cid }),
            );
            obj.insert(
                "noteForm".into(),
                serde_json::json!({ "text": "", "contact_id": cid }),
            );
            obj.insert("interactions".into(), serde_json::json!(interaction_rows));
            obj.insert("mailingHistory".into(), history_data);
        }
    }

    let container_nodes = Container::new()
        .id("contact-form-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }
    for (id, component) in extra_descendants {
        nodes.insert(id, component);
    }

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root: "contact-form-root".into(),
            nodes,
            data: merged_data,
        }),
        nav_active_patch("contacts"),
    ])
}

/// Build a `PatchMessage` that marks `<active_slug>` as the active nav item and
/// clears all others. Emitted alongside every screen Render so the sidebar's
/// `NavItem` active indicators (bound to `/nav/active/<slug>`) stay in sync
/// with the currently-visible screen. Per D-B13.
fn nav_active_patch(active_slug: &str) -> marionette_protocol::ProtocolMessage {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;
    let slugs = ["home", "contacts", "companies", "users", "audit"];
    let ops: Vec<PatchOperation> = slugs
        .iter()
        .map(|s| PatchOperation::Set {
            path: format!("/nav/active/{s}"),
            value: serde_json::json!(*s == active_slug),
        })
        .collect();
    marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "main".into(),
        patch: ops,
    })
}

/// Handle the `contact_save` action: create or update a contact.
pub async fn handle_contact_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::{NotSet, Set};

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<ContactSavePayload>::from_context(&ctx)?;
    let data = payload.0.contact_form;

    // Validate required fields
    if data.name.trim().is_empty() {
        return Err(ActionError::BadPayload(
            "Contact name is required".into(),
        ));
    }
    if data.email.trim().is_empty() {
        return Err(ActionError::BadPayload("Email is required".into()));
    }
    if !data.email.contains('@') {
        return Err(ActionError::BadPayload(
            "Invalid email format".into(),
        ));
    }

    // Parse optional company FK
    let company_id: Option<i32> = data
        .company
        .as_deref()
        .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });

    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);

    match data.id {
        None => {
            // Create mode
            let contact_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "phone": &data.phone,
                "title": &data.title,
                "company": company_id,
            });

            let new_contact = contact::ActiveModel {
                contact_id: NotSet,
                contact_name: Set(data.name),
                contact_email: Set(data.email),
                contact_phone: Set(data.phone),
                contact_title: Set(data.title),
                contact_company: Set(company_id),
                contact_country: NotSet,
                contact_notes: NotSet,
                contact_opt_in: NotSet,
                contact_created_at: NotSet,
                contact_updated_at: NotSet,
            };
            let inserted = new_contact
                .insert(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            crate::audit::record_audit(
                &*db.0,
                caller_id,
                "contact",
                inserted.contact_id,
                "create",
                contact_json,
            )
            .await?;
        }
        Some(cid) => {
            // Edit mode: fetch existing contact
            let found = contact::Entity::find_by_id(cid)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;

            let old_email = found.contact_email.clone();

            let old_json = serde_json::json!({
                "name": found.contact_name,
                "email": found.contact_email,
                "phone": found.contact_phone,
                "title": found.contact_title,
                "company": found.contact_company,
            });

            let mut active: contact::ActiveModel = found.into();
            active.contact_name = Set(data.name.clone());
            active.contact_email = Set(data.email.clone());
            active.contact_phone = Set(data.phone.clone());
            active.contact_title = Set(data.title.clone());
            active.contact_company = Set(company_id);
            active.contact_updated_at = Set(now_sqlite());

            active
                .update(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let new_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "phone": &data.phone,
                "title": &data.title,
                "company": company_id,
            });
            let changes = crate::audit::compute_changes(&old_json, &new_json);
            crate::audit::record_audit(&*db.0, caller_id, "contact", cid, "update", changes)
                .await?;

            // Propagate email change to Listmonk if subscriber exists
            if old_email != data.email {
                if let Ok(Some(sync_record)) = listmonk_sync::Entity::find()
                    .filter(listmonk_sync::Column::ListmonkSyncContact.eq(cid))
                    .one(&*db.0)
                    .await
                {
                    if let Some(subscriber_id) = sync_record.listmonk_sync_subscriber_id {
                        if let Some(client) = super::listmonk::get_listmonk_client() {
                            let name = format!("{}", &data.name);
                            if let Err(e) =
                                client.update_subscriber(subscriber_id, &data.email, &name).await
                            {
                                tracing::warn!(
                                    contact_id = cid,
                                    subscriber_id,
                                    error = %e,
                                    "Failed to update Listmonk subscriber email"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // After save, re-render the contact list
    render_contact_list(&ctx).await
}

/// Handle the `contact_delete` action: delete a contact by ID and re-render the list.
pub async fn handle_contact_delete(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = Payload::<ContactIdPayload>::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;

    // Find contact
    let found = contact::Entity::find_by_id(payload.0.contact_id)
        .one(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;

    let deleted_json = serde_json::json!({
        "name": found.contact_name,
        "email": found.contact_email,
        "phone": found.contact_phone,
        "title": found.contact_title,
        "company": found.contact_company,
    });

    // Blocklist subscriber in Listmonk if synced (best-effort)
    let deleted_id = found.contact_id;
    if let Ok(Some(sync_record)) = listmonk_sync::Entity::find()
        .filter(listmonk_sync::Column::ListmonkSyncContact.eq(deleted_id))
        .one(&*db.0)
        .await
    {
        if let Some(subscriber_id) = sync_record.listmonk_sync_subscriber_id {
            if let Some(client) = super::listmonk::get_listmonk_client() {
                if let Err(e) = client.blocklist_subscriber(subscriber_id).await {
                    tracing::warn!(
                        contact_id = deleted_id,
                        subscriber_id,
                        error = %e,
                        "Failed to blocklist Listmonk subscriber on delete (best-effort)"
                    );
                }
            }
        }
    }

    // Delete
    found
        .delete(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Record audit entry
    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "contact",
        deleted_id,
        "delete",
        deleted_json,
    )
    .await?;

    // Re-render the list
    render_contact_list(&ctx).await
}

// -- Tag management handlers --

/// Inner form data for adding a tag to a contact.
#[derive(Deserialize)]
struct TagFormData {
    contact_id: i32,
    name: String,
}

/// Payload wrapper: the frontend sends all surface data, with tag form
/// fields nested under `tagForm`.
#[derive(Deserialize)]
struct TagSavePayload {
    #[serde(rename = "tagForm")]
    tag_form: TagFormData,
}

/// Payload for removing a tag from a contact.
#[derive(Deserialize)]
struct TagRemovePayload {
    contact_id: i32,
    tag_id: i32,
}

/// Find or create a tag by name, return its ID.
async fn find_or_create_tag(
    db: &sea_orm::DatabaseConnection,
    name: &str,
) -> Result<i32, ActionError> {
    use sea_orm::ActiveValue::{NotSet, Set};

    let trimmed = name.trim();
    if let Some(existing) = tag::Entity::find()
        .filter(tag::Column::TagName.eq(trimmed))
        .one(db)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
    {
        return Ok(existing.tag_id);
    }

    let new_tag = tag::ActiveModel {
        tag_id: NotSet,
        tag_name: Set(trimmed.to_owned()),
    };
    let result = new_tag
        .insert(db)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    Ok(result.tag_id)
}

/// Handle the `contact_tag_save` action: add a tag to a contact (auto-create if new).
pub async fn handle_contact_tag_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::Set;

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<TagSavePayload>::from_context(&ctx)?;
    let data = payload.0.tag_form;

    let tag_name = data.name.trim().to_owned();
    if tag_name.is_empty() {
        return Err(ActionError::BadPayload("Tag name is required".into()));
    }

    let tag_id = find_or_create_tag(&*db.0, &tag_name).await?;

    // Insert contact_tag link; ignore unique constraint violation (tag already applied)
    let link = contact_tag::ActiveModel {
        contact_tag_contact: Set(data.contact_id),
        contact_tag_tag: Set(tag_id),
    };
    if let Err(e) = link.insert(&*db.0).await {
        let msg = e.to_string();
        // Unique constraint violation means the tag is already applied -- no-op
        if !msg.contains("UNIQUE") {
            return Err(ActionError::Internal(msg));
        }
    }

    // Audit
    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "contact_tag",
        data.contact_id,
        "create",
        serde_json::json!({ "contact_id": data.contact_id, "tag": tag_name }),
    )
    .await?;

    // Re-render the contact form
    let mut form_action = ctx.action.clone();
    form_action.payload = Some(serde_json::json!({ "contact_id": data.contact_id }));
    let form_ctx = HandlerContext {
        action: form_action,
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    };
    handle_contact_form(form_ctx).await
}

/// Handle the `contact_tag_remove` action: remove a tag from a contact.
pub async fn handle_contact_tag_remove(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<TagRemovePayload>::from_context(&ctx)?;
    let data = payload.0;

    // Delete the contact_tag link
    contact_tag::Entity::delete_many()
        .filter(contact_tag::Column::ContactTagContact.eq(data.contact_id))
        .filter(contact_tag::Column::ContactTagTag.eq(data.tag_id))
        .exec(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Audit
    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "contact_tag",
        data.contact_id,
        "delete",
        serde_json::json!({ "contact_id": data.contact_id, "tag_id": data.tag_id }),
    )
    .await?;

    // Re-render the contact form
    let mut form_action = ctx.action.clone();
    form_action.payload = Some(serde_json::json!({ "contact_id": data.contact_id }));
    let form_ctx = HandlerContext {
        action: form_action,
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    };
    handle_contact_form(form_ctx).await
}

// -----------------------------------------------------------------------------
// Phase 12 Plan 08 — country-select node-patch demo (D-A6, D-B15)
// -----------------------------------------------------------------------------

/// Handle the contact form's country-select change: swap country-specific
/// fields in place via node patches (D-A6 focus-preservation demo) and
/// insert a dismissable toast on the `toasts` sub-surface (D-B15 toast
/// lifecycle demo).
///
/// The handler returns two atomic `PatchMessage`s in order:
///
/// 1. `surface: "content"` — a mix of `Set` (confirm new country value),
///    `RemoveChild` + `DeleteNode` (tear down any previously-inserted
///    country-specific field), and `SetNode` + `InsertChild` (insert the
///    new country-specific field). Deleting a non-existent node is a
///    no-op in the frontend store, so the three candidate IDs
///    (`contact-ch-canton`, `contact-us-state`, `contact-de-bundesland`)
///    are always cleaned up before insertion.
/// 2. `surface: "toasts"` — `RemoveChild` + `DeleteNode` to wipe any
///    stale toast with the same id, then `SetNode` + `InsertChild` to
///    add a new toast node. The toast is a `Button` (so the click action
///    is actually dispatched — `Heading` ignores `action`) that triggers
///    `dismiss_toast` to close the D-B15 lifecycle.
///
/// # Errors
///
/// Returns `ActionError` only if a downstream extractor fails; the
/// patch-building itself is infallible.
#[allow(clippy::too_many_lines)]
pub async fn handle_contact_country_change(ctx: HandlerContext) -> ActionResult {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;
    use marionette_protocol::{Component, ComponentAction};

    // Extract the new country value. The Button / SelectInput payload
    // pattern embeds the full surface data under the surface root, so the
    // form values land at `payload.contactForm.country`. Fall back to
    // `payload.country` for robustness under manual/scripted dispatches.
    let payload = ctx.action.payload.clone().unwrap_or_default();
    let country = payload
        .get("contactForm")
        .and_then(|v| v.get("country"))
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("country").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    // -------- Content surface patch --------
    let mut ops: Vec<PatchOperation> = Vec::new();

    // (1) Authoritative write of the new country value — confirms the
    // optimistic change the SelectInput already wrote locally.
    ops.push(PatchOperation::Set {
        path: "/contactForm/country".into(),
        value: serde_json::json!(country),
    });

    // (2) Tear down any previously-inserted country-specific fields.
    // `remove-child` and `delete-node` are no-ops if the target doesn't
    // exist, so unconditionally clearing all three candidate IDs is safe.
    //
    // Phase 14 Plan 08 (D-A2): the country-specific swap target is now
    // the `organisation-set` FieldSet (not the outer `contact-form`),
    // because the country select is composed inside the Organisation
    // fieldset after the edit-form migration.
    for id in [
        "contact-ch-canton",
        "contact-us-state",
        "contact-de-bundesland",
    ] {
        ops.push(PatchOperation::RemoveChild {
            parent: "organisation-set".into(),
            child_id: id.into(),
        });
        ops.push(PatchOperation::DeleteNode { id: id.into() });
    }

    // (3) Insert the new country-specific field. The `organisation-set`
    // FieldSet children are `[company_select, country_select]` after the
    // Plan 08 migration, so the new field slots in at index 2 (end of
    // the list, immediately after the country select).
    let insert_index: usize = 2;

    match country.as_str() {
        "CH" => {
            let (canton_id, canton_component) = Select::new(
                "Canton",
                vec![
                    SelectOption {
                        value: "ZH".into(),
                        label: "Zürich".into(),
                    },
                    SelectOption {
                        value: "BE".into(),
                        label: "Bern".into(),
                    },
                    SelectOption {
                        value: "GE".into(),
                        label: "Geneva".into(),
                    },
                ],
            )
            .id("contact-ch-canton")
            .bind("/contactForm/canton")
            .build();
            ops.push(PatchOperation::SetNode {
                id: canton_id.clone(),
                component: canton_component,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "organisation-set".into(),
                index: insert_index,
                child_id: canton_id,
            });
        }
        "US" => {
            let (state_id, state_component) = TextInput::new("State")
                .id("contact-us-state")
                .bind("/contactForm/usState")
                .build();
            ops.push(PatchOperation::SetNode {
                id: state_id.clone(),
                component: state_component,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "organisation-set".into(),
                index: insert_index,
                child_id: state_id,
            });
        }
        "DE" => {
            let (bundesland_id, bundesland_component) = TextInput::new("Bundesland")
                .id("contact-de-bundesland")
                .bind("/contactForm/bundesland")
                .build();
            ops.push(PatchOperation::SetNode {
                id: bundesland_id.clone(),
                component: bundesland_component,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "organisation-set".into(),
                index: insert_index,
                child_id: bundesland_id,
            });
        }
        _ => {}
    }

    let content_patch = ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    });

    // -------- Toasts sub-surface patch (D-B15) --------
    //
    // The toast node is a `Button` (not a `Heading`) because the Button
    // SDUI component is the one that dispatches click actions; Heading
    // ignores its `action` field. The button's label carries the D-B15
    // toast text the E2E test asserts on.
    let toast_label = format!(
        "Country set to {}",
        match country.as_str() {
            "CH" => "Switzerland",
            "US" => "United States",
            "DE" => "Germany",
            _ => "none",
        }
    );
    let mut toast_props = serde_json::Map::new();
    toast_props.insert("label".into(), serde_json::json!(toast_label));
    let toast_node = Component {
        r#type: "button".into(),
        props: Some(serde_json::Value::Object(toast_props)),
        children: None,
        bind: None,
        action: Some(ComponentAction::click("dismiss_toast")),
        visible: None,
    };
    let toasts_ops: Vec<PatchOperation> = vec![
        // Idempotent cleanup of any stale toast with the same id.
        PatchOperation::RemoveChild {
            parent: "toasts-root".into(),
            child_id: "toast-country-change".into(),
        },
        PatchOperation::DeleteNode {
            id: "toast-country-change".into(),
        },
        // Insert the new toast node.
        PatchOperation::SetNode {
            id: "toast-country-change".into(),
            component: toast_node,
        },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(),
            index: 0,
            child_id: "toast-country-change".into(),
        },
    ];
    let toasts_patch = ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "toasts".into(),
        patch: toasts_ops,
    });

    Ok(vec![content_patch, toasts_patch])
}

/// D-B15 `dismiss_toast` handler: removes the toast node with the id carried
/// in the action payload (or the fixed "toast-country-change" id if not
/// supplied). Proves that `delete-node` works on the `toasts` sub-surface.
///
/// # Errors
///
/// Infallible in practice; signature returns `ActionResult` to match the
/// handler trait.
pub async fn handle_dismiss_toast(ctx: HandlerContext) -> ActionResult {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;

    let payload = ctx.action.payload.clone().unwrap_or_default();
    let toast_id = payload
        .get("toastId")
        .and_then(|v| v.as_str())
        .unwrap_or("toast-country-change")
        .to_string();

    let ops = vec![
        PatchOperation::RemoveChild {
            parent: "toasts-root".into(),
            child_id: toast_id.clone(),
        },
        PatchOperation::DeleteNode { id: toast_id },
    ];

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "toasts".into(),
        patch: ops,
    })])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// V-07 / 13-VALIDATION.md row 7 — V5 Input Validation.
    ///
    /// Structurally-invalid payloads (e.g., `date` is a number, not an
    /// object) must be rejected by serde at deserialize time. This proves
    /// the `#[derive(Deserialize)]` on `ContactFilterParams` takes the
    /// typed path and will surface `ActionError::BadPayload` at the
    /// handler boundary when payloads don't conform to the shape.
    ///
    /// Structurally-valid date strings like `"not-a-date"` are accepted at
    /// deserialize time (serde doesn't parse the date) and flow through
    /// as parameterized SeaORM comparisons which never inject — this is
    /// the documented T-13-06-02 disposition.
    #[test]
    fn contact_filter_params_rejects_bad_date() {
        let bad_shape = json!({
            "search": "Alice",
            "date": 42
        });
        let r = serde_json::from_value::<ContactFilterParams>(bad_shape);
        assert!(
            r.is_err(),
            "expected deserialize error for malformed date-range shape"
        );

        let bad_date_string = json!({
            "search": "Alice",
            "date": { "from": "not-a-date", "to": "2026-13-01" }
        });
        let parsed: ContactFilterParams = serde_json::from_value(bad_date_string)
            .expect("strings-as-dates should deserialize; SeaORM handles bad values at query time");
        assert!(parsed.date.is_some());
    }

    /// Round-trip proof that the new `ContactFilterParams` shape accepts
    /// every field the frontend legitimately sends: search, company
    /// filter, tag filter, and the collapsed `date` date-range.
    #[test]
    fn contact_filter_params_deserializes_full_payload() {
        let payload = json!({
            "search": "Alice",
            "company_filter": "acme-inc",
            "tag_filter_text": "vip,priority",
            "date": { "from": "2026-01-01", "to": "2026-04-01" }
        });
        let parsed: ContactFilterParams =
            serde_json::from_value(payload).expect("full payload should deserialize");
        assert_eq!(parsed.search.as_deref(), Some("Alice"));
        assert_eq!(parsed.company_filter.as_deref(), Some("acme-inc"));
        assert_eq!(parsed.tag_filter_text.as_deref(), Some("vip,priority"));
        assert!(parsed.date.is_some());
        let dr = parsed.date.unwrap();
        assert_eq!(dr.from.as_deref(), Some("2026-01-01"));
        assert_eq!(dr.to.as_deref(), Some("2026-04-01"));
    }
}
