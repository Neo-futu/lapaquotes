//! Compact Outlook-style month calendar for Hong Kong public holidays.
//! Selection is bound to the Quotes page DATE filter (`quote_date` signal).

use chrono::{Datelike, Month, NaiveDate, Weekday};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolidayView {
    pub holiday_date: NaiveDate,
    pub name: String,
}

pub fn today_local() -> NaiveDate {
    #[cfg(feature = "hydrate")]
    {
        let d = js_sys::Date::new_0();
        NaiveDate::from_ymd_opt(
            d.get_full_year() as i32,
            (d.get_month() as u32) + 1,
            d.get_date() as u32,
        )
        .unwrap_or_else(|| chrono::Utc::now().date_naive())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        (chrono::Utc::now() + chrono::Duration::hours(8)).date_naive()
    }
}

fn parse_ymd(s: &str) -> Option<NaiveDate> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()
}

fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let mut m = month as i32 + delta;
    let mut y = year;
    while m > 12 {
        m -= 12;
        y += 1;
    }
    while m < 1 {
        m += 12;
        y -= 1;
    }
    (y, m as u32)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = shift_month(year, month, 1);
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

fn month_title(year: i32, month: u32) -> String {
    let name = Month::try_from(month as u8)
        .map(|m| m.name())
        .unwrap_or("");
    format!("{name} {year}")
}

/// `quote_date` is the shared Quotes DATE filter (`YYYY-MM-DD`).
#[component]
pub fn HkHolidayCalendar(quote_date: RwSignal<String>) -> impl IntoView {
    let today = today_local();
    let initial = parse_ymd(&quote_date.get_untracked()).unwrap_or(today);
    let view_year = RwSignal::new(initial.year());
    let view_month = RwSignal::new(initial.month());

    // Keep the visible month in sync when DATE is changed elsewhere (date input / default today).
    Effect::new(move |_| {
        if let Some(d) = parse_ymd(&quote_date.get()) {
            view_year.set(d.year());
            view_month.set(d.month());
        }
    });

    let holidays = Resource::new_blocking(|| (), move |_| get_hk_holidays());

    view! {
        <aside class="HkCal" aria-label="Hong Kong holiday calendar">
            <Suspense fallback=|| {
                view! { <div class="HkCal-card"><p class="HkCal-loading">"…"</p></div> }
            }>
                {move || Suspend::new(async move {
                    let list = holidays.await.unwrap_or_default();
                    let map: HashMap<NaiveDate, String> = list
                        .into_iter()
                        .map(|h| (h.holiday_date, h.name))
                        .collect();
                    view! {
                        <HkCalInner
                            today
                            quote_date
                            view_year
                            view_month
                            holidays=map
                        />
                    }
                })}
            </Suspense>
        </aside>
    }
}

#[component]
fn HkCalInner(
    today: NaiveDate,
    quote_date: RwSignal<String>,
    view_year: RwSignal<i32>,
    view_month: RwSignal<u32>,
    holidays: HashMap<NaiveDate, String>,
) -> impl IntoView {
    let holidays = RwSignal::new(holidays);

    let selected = Memo::new(move |_| {
        parse_ymd(&quote_date.get()).unwrap_or(today)
    });

    let set_quote_day = move |d: NaiveDate| {
        quote_date.set(d.format("%Y-%m-%d").to_string());
    };

    let prev = move |_| {
        let (y, m) = shift_month(view_year.get(), view_month.get(), -1);
        view_year.set(y);
        view_month.set(m);
    };
    let next = move |_| {
        let (y, m) = shift_month(view_year.get(), view_month.get(), 1);
        view_year.set(y);
        view_month.set(m);
    };
    let go_today = move |_| {
        set_quote_day(today);
    };

    view! {
        <div class="HkCal-card">
            <header class="HkCal-head">
                <div class="HkCal-title-row">
                    <span class="HkCal-badge">"HK"</span>
                    <button type="button" class="HkCal-nav" on:click=prev aria-label="Previous month">
                        "‹"
                    </button>
                    <h2 class="HkCal-month">
                        {move || month_title(view_year.get(), view_month.get())}
                    </h2>
                    <button type="button" class="HkCal-nav" on:click=next aria-label="Next month">
                        "›"
                    </button>
                </div>
                <button type="button" class="HkCal-today-btn" on:click=go_today>
                    "Today"
                </button>
            </header>

            <div class="HkCal-weekdays">
                <span>"Mon"</span><span>"Tue"</span><span>"Wed"</span><span>"Thu"</span>
                <span>"Fri"</span><span>"Sat"</span><span>"Sun"</span>
            </div>

            <div class="HkCal-grid" role="grid">
                {move || {
                    let y = view_year.get();
                    let m = view_month.get();
                    let map = holidays.get();
                    let _ = selected.get(); // rebuild cells when DATE / selection changes
                    let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                    let lead = first.weekday().num_days_from_monday() as i32;
                    let dim = days_in_month(y, m) as i32;
                    (0..42)
                        .map(|i| {
                            let day_idx = i - lead + 1;
                            if day_idx < 1 || day_idx > dim {
                                return view! {
                                    <span class="HkCal-cell HkCal-cell--empty"></span>
                                }
                                .into_any();
                            }
                            let d = day_idx as u32;
                            let cell = NaiveDate::from_ymd_opt(y, m, d).unwrap();
                            let is_today = cell == today;
                            let is_holiday = map.contains_key(&cell);
                            let is_weekend = matches!(cell.weekday(), Weekday::Sat | Weekday::Sun);
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let mut c = String::from("HkCal-cell");
                                        if is_today {
                                            c.push_str(" HkCal-cell--today");
                                        }
                                        if selected.get() == cell {
                                            c.push_str(" HkCal-cell--selected");
                                        }
                                        if is_holiday {
                                            c.push_str(" HkCal-cell--holiday");
                                        }
                                        if is_weekend {
                                            c.push_str(" HkCal-cell--weekend");
                                        }
                                        c
                                    }
                                    on:click=move |_| set_quote_day(cell)
                                >
                                    <span class="HkCal-daynum">{d.to_string()}</span>
                                    <Show when=move || is_holiday>
                                        <span class="HkCal-dot" aria-hidden="true"></span>
                                    </Show>
                                </button>
                            }
                            .into_any()
                        })
                        .collect_view()
                }}
            </div>

            <footer class="HkCal-detail">
                {move || {
                    let day = selected.get();
                    let name = holidays.with(|h| h.get(&day).cloned());
                    let date_label = day.format("%A, %d %b %Y").to_string();
                    let is_today = day == today;
                    view! {
                        <div class="HkCal-detail-date">
                            {date_label}
                            <Show when=move || is_today>
                                <span class="HkCal-pill">"Today"</span>
                            </Show>
                        </div>
                        {if let Some(n) = name {
                            view! {
                                <div class="HkCal-detail-holiday">
                                    <span class="HkCal-pill HkCal-pill--holiday">"Holiday"</span>
                                    " "
                                    {n}
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div class="HkCal-detail-none">"No Hong Kong public holiday"</div>
                            }
                            .into_any()
                        }}
                    }
                }}
            </footer>
        </div>
    }
}

#[server(GetHkHolidays, "/api")]
pub async fn get_hk_holidays() -> Result<Vec<HolidayView>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let region = Some("HK".to_string());
    let rows = cornucopia::queries::holiday::holidays_by_region()
        .bind(&db, &region)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "HK holidays list"))?;
    Ok(rows
        .into_iter()
        .map(|r| HolidayView {
            holiday_date: r.holiday_date,
            name: r.name,
        })
        .collect())
}
