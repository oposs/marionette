# Feature Research

**Domain:** SDUI Protocol (OpenSDUI) + Simple CRM Demo Application
**Researched:** 2026-01-23
**Confidence:** MEDIUM (verified against multiple sources and existing CONCEPT.md)

---

## Part 1: SDUI Framework Features

Server-Driven UI is a design pattern, not a library. OpenSDUI defines an open protocol specification; the features below represent what mature SDUI implementations provide.

### Table Stakes (Users Expect These)

Features that any serious SDUI framework must have. Missing these = framework feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Component Rendering** | Core purpose of SDUI - render typed components from server | HIGH | Must handle text, images, buttons, containers at minimum |
| **Data Binding** | Components must reflect data state | MEDIUM | JSON Pointer paths (RFC 6901) per CONCEPT.md |
| **Layout Containers** | UI needs structure (column, row, grid) | MEDIUM | DivKit: container, gallery, grid, pager, tabs |
| **Action Handling** | User interactions must reach server | MEDIUM | Button clicks, form submits, navigation |
| **Form Inputs** | Data entry is fundamental to apps | MEDIUM | Text, select, checkbox minimum; DivKit: input, select, slider |
| **Navigation** | Multi-screen apps require routing | MEDIUM | Server-driven routes with deep linking support |
| **Error Display** | Validation/errors must show to user | LOW | CONCEPT.md: "Errors as data" - bind to error paths |
| **Loading States** | Async ops need user feedback | LOW | Bind spinner to boolean data path per CONCEPT.md |
| **Styling/Theming** | Consistent visual appearance | MEDIUM | Props for colors, spacing, typography |
| **Flat Component Model** | Patching and streaming | MEDIUM | Adjacency list pattern from A2UI per CONCEPT.md |

### Differentiators (Competitive Advantage)

Features that set SDUI frameworks apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Open Protocol Spec** | Vendor-neutral, multiple implementations | HIGH | OpenSDUI's core differentiator vs proprietary systems |
| **Real-time Updates (WebSocket)** | Push UI changes without polling | MEDIUM | CONCEPT.md supports REST + WebSocket |
| **Optimistic Updates** | Responsive feel for slow networks | MEDIUM | CONCEPT.md: "action can include optimistic patch" |
| **Template System** | Reusable component patterns | MEDIUM | DivKit emphasizes "encapsulation and reuse" |
| **Conditional Visibility** | Server controls what shows | LOW | Bind to boolean path per CONCEPT.md |
| **Inter-widget Communication** | Field A affects Field B | MEDIUM | "Filling age field lights up submit button" |
| **Multi-surface Rendering** | Main, modal, toast, sidebar | LOW | CONCEPT.md: named surfaces |
| **Streaming/Incremental** | Large UIs load progressively | HIGH | Adjacency list enables streaming nodes |
| **State Management** | Client-side state for 60fps interactions | MEDIUM | DivKit: states, animations, transitions |
| **LLM-friendly JSON** | AI can generate UI descriptions | LOW | Flat structure easier for LLMs per CONCEPT.md |
| **Version Negotiation** | Handle cached/stale frontends | LOW | CONCEPT.md mentions version in initial message |
| **Accessibility (ARIA)** | Semantic components infer ARIA | MEDIUM | Frontend infers from component types |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Component Schema Negotiation** | "Frontend tells server what it supports" | Adds complexity; you control both sides | Document your component library; deploy together |
| **Pixel-perfect Layout Control** | "Backend specifies exact positions" | Breaks responsive design; platform differences | Semantic layout props; let frontend handle details |
| **Embedded Business Logic** | "Validation rules in JSON" | Duplicates server logic; security hole | Server validates authoritatively; client hints only |
| **Generic Component Types** | "Render any HTML/native code" | Security nightmare; defeats SDUI benefits | Curated component library with extension points |
| **Real-time Everything** | "All updates via WebSocket" | Complexity without value for most screens | REST for requests; WebSocket for push events |
| **Deep Nested Structures** | "Natural tree representation" | Hard to patch; streaming unfriendly | Flat adjacency list per CONCEPT.md |
| **Array Index Binding** | "Bind to /users/0/name" | Unstable when items added/removed | Keyed objects with stable IDs per CONCEPT.md |

---

## Part 2: Simple CRM Features

Features for a simple CRM (the demo application). Focus: contacts, companies, interactions, with Listmonk integration.

### Table Stakes (Users Expect These)

Features users assume exist. Missing = CRM feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Contact Records** | Core CRM purpose | MEDIUM | Name, email, phone, company link, custom fields |
| **Company Records** | B2B relationships | MEDIUM | Name, address, linked contacts, industry |
| **Interaction History** | "What did we discuss?" | MEDIUM | Notes, calls, emails, meetings per contact |
| **Search** | Find contacts quickly | LOW | Search by name, email, company |
| **Notes** | Free-form context on records | LOW | Text notes with timestamps |
| **Tags/Labels** | Categorize contacts | LOW | Flexible classification system |
| **Basic Filtering** | View subsets of data | LOW | Filter by company, tag, date range |
| **Data Tables** | View lists of records | MEDIUM | Sortable, paginated tables |
| **Record Detail Views** | See full contact info | MEDIUM | Form-based detail view with edit |
| **Basic User Auth** | Multi-user access | MEDIUM | Login, sessions, per CONCEPT.md roles |

### Differentiators (Competitive Advantage)

Features that set the CRM apart. Valuable but not essential.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Listmonk Integration** | Newsletter management built-in | MEDIUM | Sync subscribers, manage lists, track campaigns |
| **Server-Driven UI** | Backend controls all UI | HIGH | The entire point - proves OpenSDUI works |
| **Activity Timeline** | Chronological interaction view | LOW | Unified view of all touchpoints |
| **Bulk Operations** | Manage many contacts at once | MEDIUM | Bulk tag, export, delete |
| **Import/Export** | Get data in/out easily | MEDIUM | CSV import, export functionality |
| **Custom Fields** | Adapt to specific needs | MEDIUM | User-defined fields on contacts/companies |
| **Email Integration** | Log emails automatically | HIGH | Connect to email provider, auto-log |
| **Real-time Updates** | Multi-user sees changes | MEDIUM | WebSocket push when others edit |
| **Role-based Access** | Different permissions | MEDIUM | Admin vs user vs read-only |
| **Audit Trail** | Who changed what when | MEDIUM | Track modifications for compliance |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but add complexity without proportional value for a *simple* CRM.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Sales Pipeline/Deals** | "I need to track opportunities" | Adds significant complexity; scope creep | Keep CRM focused on contacts; separate tool for sales |
| **Full Email Client** | "Send emails from CRM" | Hard to do well; deliverability issues | Link to email; log interactions instead |
| **Calendar Integration** | "See meetings in CRM" | Complex OAuth, sync issues | Log meeting notes manually |
| **AI Auto-Logging** | "Automatically parse emails" | Unreliable; privacy concerns | Simple manual logging with good UX |
| **Social Media Tracking** | "Pull in LinkedIn data" | API restrictions; privacy/legal issues | Manual entry of relevant info |
| **Complex Workflows** | "Automated sequences" | Requires workflow engine; maintenance burden | Focus on simple task reminders |
| **Built-in Calling** | "Click to dial" | Telecom complexity; regulations | Link to tel: protocol; external tools |
| **Lead Scoring** | "Rank contact priority" | Requires ML/algorithms; hard to tune | Simple manual priority tags |

---

## Feature Dependencies

### SDUI Protocol Dependencies

```
[Component Rendering]
    |-- requires --> [Data Binding]
    |-- requires --> [Layout Containers]

[Form Inputs]
    |-- requires --> [Data Binding]
    |-- requires --> [Action Handling]

[Error Display]
    |-- requires --> [Data Binding] (errors as data)

[Navigation]
    |-- requires --> [Action Handling]
    |-- requires --> [Multi-surface Rendering]

[Optimistic Updates]
    |-- requires --> [Action Handling]
    |-- requires --> [Data Binding]

[Real-time Updates]
    |-- requires --> [Data Binding] (patches)
    |-- enhances --> [Multi-user scenarios]
```

### CRM Feature Dependencies

```
[Contact Records]
    |-- enhanced by --> [Company Records] (linking)
    |-- enhanced by --> [Tags/Labels]

[Company Records]
    |-- requires --> [Contact Records] (for linking)

[Interaction History]
    |-- requires --> [Contact Records]
    |-- requires --> [Notes]

[Listmonk Integration]
    |-- requires --> [Contact Records] (subscriber sync)
    |-- requires --> [Tags/Labels] (list mapping)

[Search]
    |-- requires --> [Contact Records]
    |-- requires --> [Company Records]

[Bulk Operations]
    |-- requires --> [Data Tables] (selection UI)
    |-- requires --> [Tags/Labels] (bulk tagging)
```

### Dependency Notes

- **Data Binding is foundational:** Nearly every SDUI feature depends on binding components to data paths. Build this first and build it right.
- **Contact Records anchor CRM:** Companies, interactions, and integrations all reference contacts. This is the core entity.
- **Listmonk Integration requires stable contacts:** Must have reliable contact management before adding newsletter sync.
- **Real-time updates are optional:** Can ship MVP with REST-only; add WebSocket later.

---

## MVP Definition

### Launch With (v1) - SDUI Protocol

Minimum viable protocol implementation.

- [x] Component Rendering - text, container, button, image
- [x] Data Binding - JSON Pointer paths, two-way for inputs
- [x] Form Inputs - text-input, select, checkbox, button
- [x] Action Handling - click, submit, navigate actions
- [x] Layout Containers - container, grid/flex layout
- [x] Error Display - bind to error data paths
- [x] Loading States - bind to boolean loading paths
- [x] Single Surface - main content area

### Launch With (v1) - CRM Demo

Minimum viable CRM to demonstrate OpenSDUI.

- [x] Contact Records - create, read, update, delete
- [x] Company Records - create, read, update, delete, link contacts
- [x] Notes - add notes to contacts/companies
- [x] Data Tables - list contacts and companies with pagination
- [x] Search - find by name, email
- [x] Basic Auth - login, user sessions
- [x] Record Detail Views - view/edit contact and company details

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] Interaction History - when "notes" proves useful
- [ ] Tags/Labels - when categorization need emerges
- [ ] Multi-surface (modal, toast) - when UX demands it
- [ ] Listmonk Integration - after contacts are stable
- [ ] Navigation (routes) - when app has multiple distinct screens
- [ ] Basic Filtering - when list views get crowded

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Real-time Updates (WebSocket) - adds complexity
- [ ] Optimistic Updates - refinement, not essential
- [ ] Custom Fields - wait for user feedback
- [ ] Bulk Operations - scale feature
- [ ] Import/Export - data portability
- [ ] Role-based Access - when multi-org needs arise
- [ ] Template System - when patterns emerge
- [ ] Audit Trail - compliance requirement, not MVP

---

## Feature Prioritization Matrix

### SDUI Protocol Features

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Component Rendering | HIGH | HIGH | P1 |
| Data Binding | HIGH | MEDIUM | P1 |
| Form Inputs | HIGH | MEDIUM | P1 |
| Action Handling | HIGH | MEDIUM | P1 |
| Layout Containers | HIGH | MEDIUM | P1 |
| Error Display | MEDIUM | LOW | P1 |
| Loading States | MEDIUM | LOW | P1 |
| Navigation | MEDIUM | MEDIUM | P2 |
| Multi-surface Rendering | MEDIUM | LOW | P2 |
| Real-time Updates | LOW | MEDIUM | P3 |
| Optimistic Updates | LOW | MEDIUM | P3 |
| Template System | MEDIUM | HIGH | P3 |

### CRM Demo Features

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Contact Records | HIGH | MEDIUM | P1 |
| Company Records | HIGH | MEDIUM | P1 |
| Data Tables | HIGH | MEDIUM | P1 |
| Record Detail Views | HIGH | MEDIUM | P1 |
| Basic Auth | HIGH | MEDIUM | P1 |
| Notes | MEDIUM | LOW | P1 |
| Search | MEDIUM | LOW | P1 |
| Interaction History | MEDIUM | MEDIUM | P2 |
| Tags/Labels | MEDIUM | LOW | P2 |
| Basic Filtering | MEDIUM | LOW | P2 |
| Listmonk Integration | MEDIUM | MEDIUM | P2 |
| Bulk Operations | LOW | MEDIUM | P3 |
| Import/Export | LOW | MEDIUM | P3 |
| Custom Fields | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

### SDUI Frameworks

| Feature | Airbnb Ghost | DivKit (Yandex) | OpenSDUI (Proposed) |
|---------|--------------|-----------------|---------------------|
| Open Source | No (internal) | Yes (Apache 2.0) | Yes (planned) |
| Protocol Spec | No (internal) | No (implementation) | Yes (core value) |
| Platforms | iOS, Android, Web | iOS, Android, Web, Flutter | Web (Svelte), extensible |
| Component Types | Internal catalog | text, container, input, etc. | Open set (string types) |
| Data Binding | GraphQL-based | JSON templates | JSON Pointer (RFC 6901) |
| Flat Structure | Sections/Screens | Templates + cards | Adjacency list |
| Real-time | Yes | Limited | REST + WebSocket |
| Form Inputs | Yes | input, select, slider | Planned full set |
| Navigation | Custom actions | Limited | Server-driven routes |

### Simple CRMs

| Feature | Less Annoying CRM | Capsule CRM | OpenSDUI CRM Demo |
|---------|-------------------|-------------|-------------------|
| Contact Mgmt | Yes | Yes | Yes (planned) |
| Company Mgmt | Yes | Yes | Yes (planned) |
| Interaction History | Yes | Yes | Yes (planned) |
| Pipeline/Deals | Limited | Yes | No (anti-feature) |
| Email Integration | Via Zapier | Yes | Via Listmonk |
| Newsletter | No | No | Yes (Listmonk) |
| Pricing | $15/user/mo | Free tier + paid | Self-hosted/Free |
| Server-Driven UI | No | No | Yes (key differentiator) |

---

## Sources

### SDUI Frameworks
- [Airbnb Ghost Platform Deep Dive](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5) - Airbnb Engineering blog
- [DivKit GitHub](https://github.com/divkit/divkit) - Yandex's open source SDUI framework
- [DivKit Documentation](https://divkit.tech/doc) - Component types and features
- [Apollo GraphQL SDUI](https://www.apollographql.com/docs/graphos/schema-design/guides/sdui/basics) - Schema design patterns
- [SDUI Design Patterns](https://medium.com/androidiots/mastering-sdui-a-deep-dive-into-server-driven-ui-8329ad90ab44) - Technical patterns

### CRM Features
- [OnePageCRM Features](https://www.onepagecrm.com/blog/crm-features/) - 33 CRM features for small business
- [Less Annoying CRM](https://www.lessannoyingcrm.com/) - Simple CRM reference
- [Capsule CRM Contact Management](https://capsulecrm.com/features/contact-management-software/) - Contact management features
- [CRM Table Stakes 2026](https://croclub.com/data-reporting/crm-features/) - Essential features analysis
- [Simple CRM Tools 2026](https://www.bigcontacts.com/blog/simple-crm/) - Simple CRM comparison

### Integration
- [Listmonk](https://listmonk.app/) - Self-hosted newsletter manager
- [Listmonk GitHub](https://github.com/knadh/listmonk) - API and features

### Architecture Patterns
- [SDUI Real-time Updates](https://medium.com/@dimakoua/unlocking-the-power-of-server-driven-ui-building-dynamic-configurable-apps-16a9f5bdf95a) - WebSocket patterns
- [SDUI Data Binding](https://medium.com/@dfs.techblog/sdui-series-basics-and-design-3f324a9cb0cb) - Data binding patterns

---
*Feature research for: OpenSDUI Protocol + Marionette CRM Demo*
*Researched: 2026-01-23*
