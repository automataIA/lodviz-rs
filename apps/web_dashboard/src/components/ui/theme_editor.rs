use leptos::prelude::*;
use lodviz_core::core::theme::{ChartConfig, ChartTheme, GridStyle};

#[component]
fn ColorPicker(
    #[prop(into)] label: String,
    #[prop(into)] color: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label py-0">
                <span class="label-text text-xs">{label}</span>
            </label>
            <div class="flex flex-col gap-1">
                // Hex Color Input (RGB)
                <div class="flex items-center gap-2">
                    <input
                        type="color"
                        class="input input-ghost p-0 h-6 w-8 min-w-0"
                        prop:value=move || {
                            let full_hex = color.get();
                            if full_hex.len() >= 7 { full_hex[0..7].to_string() } else { full_hex }
                        }
                        on:input=move |ev| {
                            let rgb = event_target_value(&ev);
                            let current = color.get();
                            let alpha = if current.len() == 9 {
                                current[7..9].to_string()
                            } else {
                                "ff".to_string()
                            };
                            on_change.run(format!("{}{}", rgb, alpha));
                        }
                    />
                    <span class="text-xs opacity-70 font-mono">{move || color.get()}</span>
                </div>

                // Opacity Slider (Alpha)
                <div class="flex items-center gap-2">
                    <span class="text-[10px] opacity-60 w-8">"Opacity"</span>
                    <input
                        type="range"
                        min="0"
                        max="255"
                        class="range range-xs range-info flex-1"
                        prop:value=move || {
                            let hex = color.get();
                            if hex.len() == 9 {
                                u8::from_str_radix(&hex[7..9], 16).unwrap_or(255) as i32
                            } else {
                                255
                            }
                        }
                        on:input=move |ev| {
                            if let Ok(alpha_val) = event_target_value(&ev).parse::<u8>() {
                                let current = color.get();
                                let rgb = if current.len() >= 7 {
                                    &current[0..7]
                                } else {
                                    "#000000"
                                };
                                on_change.run(format!("{}{:02x}", rgb, alpha_val));
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ThemeEditor(#[prop(into)] config: RwSignal<ChartConfig>) -> impl IntoView {
    // Local buffer for edits
    let local_config = RwSignal::new(config.get_untracked());

    // Effect to reset local buffer if the upstream config changes identity (handled by parent keying usually,
    // but good practice to allow reactive syncing if we want to support external updates).
    // Actually, distinct from keying, if `config` signal *content* changes from outside (e.g. another editor),
    // we might want `local_config` to update?
    // For now, let's rely on the "Save" model where we diverge.

    let on_save = move |_| {
        config.set(local_config.get());
    };

    view! {
        <div class="card bg-base-100 shadow-xl compact w-full max-w-sm">
            <div class="card-body p-4">
                <h2 class="card-title text-sm mb-2">"Chart Styling"</h2>

                // Title Input
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Title"</span>
                    </label>
                    <input
                        type="text"
                        class="input input-bordered input-sm w-full"
                        placeholder="Chart Title"
                        prop:value=move || local_config.get().title.unwrap_or_default()
                        on:input=move |ev| {
                            local_config.update(|c| c.title = Some(event_target_value(&ev)));
                        }
                    />
                </div>

                // Grid Controls
                <div class="form-control w-full">
                    <label class="label cursor-pointer py-1">
                        <span class="label-text text-xs">"Show X Grid"</span>
                        <input
                            type="checkbox"
                            class="toggle toggle-primary toggle-sm"
                            prop:checked=move || {
                                local_config.get().grid.as_ref().is_none_or(|g| g.show_x)
                            }
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                local_config
                                    .update(|c| {
                                        let g = c.grid.get_or_insert_with(GridStyle::default);
                                        g.show_x = checked;
                                    });
                            }
                        />
                    </label>
                </div>
                <div class="form-control w-full">
                    <label class="label cursor-pointer py-1">
                        <span class="label-text text-xs">"Show Y Grid"</span>
                        <input
                            type="checkbox"
                            class="toggle toggle-primary toggle-sm"
                            prop:checked=move || {
                                local_config.get().grid.as_ref().is_none_or(|g| g.show_y)
                            }
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                local_config
                                    .update(|c| {
                                        let g = c.grid.get_or_insert_with(GridStyle::default);
                                        g.show_y = checked;
                                    });
                            }
                        />
                    </label>
                </div>

                // Grid Opacity
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Grid Opacity"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.2}",
                                    local_config.get().grid.as_ref().map_or(0.3, |g| g.opacity),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.05"
                        class="range range-xs range-primary"
                        prop:value=move || {
                            local_config.get().grid.as_ref().map_or(0.3, |g| g.opacity)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        let g = c.grid.get_or_insert_with(GridStyle::default);
                                        g.opacity = val;
                                    });
                            }
                        }
                    />
                </div>

                // Grid Width
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Grid Width"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.1}px",
                                    local_config.get().grid.as_ref().map_or(0.5, |g| g.width),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="0.5"
                        max="5"
                        step="0.5"
                        class="range range-xs range-primary"
                        prop:value=move || {
                            local_config.get().grid.as_ref().map_or(0.5, |g| g.width)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        let g = c.grid.get_or_insert_with(GridStyle::default);
                                        g.width = val;
                                    });
                            }
                        }
                    />
                </div>

                // Grid Dash Pattern
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Grid Dash"</span>
                    </label>
                    <select
                        class="select select-bordered select-sm w-full"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            local_config
                                .update(|c| {
                                    let g = c.grid.get_or_insert_with(GridStyle::default);
                                    g.dash = match val.as_str() {
                                        "dashed" => Some("4,4".to_string()),
                                        "dotted" => Some("2,2".to_string()),
                                        _ => None,
                                    };
                                });
                        }
                    >
                        <option
                            value="solid"
                            selected=move || {
                                local_config.get().grid.as_ref().is_none_or(|g| g.dash.is_none())
                            }
                        >
                            "Solid"
                        </option>
                        <option
                            value="dashed"
                            selected=move || {
                                local_config.get().grid.as_ref().and_then(|g| g.dash.as_deref())
                                    == Some("4,4")
                            }
                        >
                            "Dashed"
                        </option>
                        <option
                            value="dotted"
                            selected=move || {
                                local_config.get().grid.as_ref().and_then(|g| g.dash.as_deref())
                                    == Some("2,2")
                            }
                        >
                            "Dotted"
                        </option>
                    </select>
                </div>

                // Font Size Range
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Font Size"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.0}px",
                                    local_config
                                        .get()
                                        .theme
                                        .as_ref()
                                        .map(|t| t.font_size)
                                        .unwrap_or(12.0),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="8"
                        max="24"
                        class="range range-xs range-primary"
                        prop:value=move || {
                            local_config.get().theme.as_ref().map(|t| t.font_size).unwrap_or(12.0)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        if c.theme.is_none() {
                                            c.theme = Some(ChartTheme::default());
                                        }
                                        if let Some(t) = c.theme.as_mut() {
                                            t.font_size = val;
                                        }
                                    });
                            }
                        }
                    />
                </div>

                // Axis Font Size Range
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Axis Font Size"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.0}px",
                                    local_config
                                        .get()
                                        .theme
                                        .as_ref()
                                        .map(|t| t.axis_font_size)
                                        .unwrap_or(10.0),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="8"
                        max="24"
                        class="range range-xs range-primary"
                        prop:value=move || {
                            local_config
                                .get()
                                .theme
                                .as_ref()
                                .map(|t| t.axis_font_size)
                                .unwrap_or(10.0)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        if c.theme.is_none() {
                                            c.theme = Some(ChartTheme::default());
                                        }
                                        if let Some(t) = c.theme.as_mut() {
                                            t.axis_font_size = val;
                                        }
                                    });
                            }
                        }
                    />
                </div>

                // Point Radius Range
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Point Radius"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.1}px",
                                    local_config
                                        .get()
                                        .theme
                                        .as_ref()
                                        .map(|t| t.point_radius)
                                        .unwrap_or(3.0),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="1"
                        max="10"
                        step="0.5"
                        class="range range-xs range-secondary"
                        prop:value=move || {
                            local_config.get().theme.as_ref().map(|t| t.point_radius).unwrap_or(3.0)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        if c.theme.is_none() {
                                            c.theme = Some(ChartTheme::default());
                                        }
                                        if let Some(t) = c.theme.as_mut() {
                                            t.point_radius = val;
                                        }
                                    });
                            }
                        }
                    />
                </div>

                // Stroke Width Range
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Stroke Width"</span>
                        <span class="label-text-alt text-xs">
                            {move || {
                                format!(
                                    "{:.1}px",
                                    local_config
                                        .get()
                                        .theme
                                        .as_ref()
                                        .map(|t| t.stroke_width)
                                        .unwrap_or(2.0),
                                )
                            }}
                        </span>
                    </label>
                    <input
                        type="range"
                        min="0.5"
                        max="10"
                        step="0.5"
                        class="range range-xs range-accent"
                        prop:value=move || {
                            local_config.get().theme.as_ref().map(|t| t.stroke_width).unwrap_or(2.0)
                        }
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                local_config
                                    .update(|c| {
                                        if c.theme.is_none() {
                                            c.theme = Some(ChartTheme::default());
                                        }
                                        if let Some(t) = c.theme.as_mut() {
                                            t.stroke_width = val;
                                        }
                                    });
                            }
                        }
                    />
                </div>

                <div class="divider my-1"></div>
                <h3 class="text-xs font-bold uppercase opacity-50 mb-2">"Legend"</h3>

                // Show Legend toggle
                <div class="form-control w-full">
                    <label class="label py-1 cursor-pointer">
                        <span class="label-text text-xs">"Show Legend"</span>
                        <input
                            type="checkbox"
                            class="toggle toggle-primary toggle-sm"
                            prop:checked=move || local_config.get().show_legend.unwrap_or(true)
                            on:change=move |ev| {
                                let val = event_target_checked(&ev);
                                local_config
                                    .update(|c| {
                                        c.show_legend = Some(val);
                                    });
                            }
                        />
                    </label>
                </div>

                // Legend Position select
                <div class="form-control w-full">
                    <label class="label py-1">
                        <span class="label-text text-xs">"Legend Position"</span>
                    </label>
                    <select
                        class="select select-bordered select-sm w-full"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            local_config
                                .update(|c| {
                                    c.legend_outside = Some(val == "outside");
                                });
                        }
                    >
                        <option
                            value="inside"
                            selected=move || !local_config.get().legend_outside.unwrap_or(false)
                        >
                            "Inside (overlay)"
                        </option>
                        <option
                            value="outside"
                            selected=move || local_config.get().legend_outside.unwrap_or(false)
                        >
                            "Outside (adjacent)"
                        </option>
                    </select>
                </div>

                <div class="divider my-1"></div>
                <h3 class="text-xs font-bold uppercase opacity-50 mb-2">"Colors"</h3>

                <div class="grid grid-cols-2 gap-2">
                    // Background Color
                    <ColorPicker
                        label="Background".to_string()
                        color=Signal::derive(move || {
                            local_config
                                .get()
                                .theme
                                .as_ref()
                                .map(|t| t.background_color.clone())
                                .unwrap_or("#ffffff".to_string())
                        })
                        on_change=Callback::new(move |new_color| {
                            local_config
                                .update(|c| {
                                    if c.theme.is_none() {
                                        c.theme = Some(ChartTheme::default());
                                    }
                                    if let Some(t) = c.theme.as_mut() {
                                        t.background_color = new_color;
                                    }
                                });
                        })
                    />

                    // Data Color
                    <ColorPicker
                        label="Data Color".to_string()
                        color=Signal::derive(move || {
                            local_config
                                .get()
                                .theme
                                .as_ref()
                                .and_then(|t| t.palette.first().cloned())
                                .unwrap_or("#5470c6".to_string())
                        })
                        on_change=Callback::new(move |new_color| {
                            local_config
                                .update(|c| {
                                    if c.theme.is_none() {
                                        c.theme = Some(ChartTheme::default());
                                    }
                                    if let Some(t) = c.theme.as_mut() {
                                        if t.palette.is_empty() {
                                            t.palette.push(new_color);
                                        } else {
                                            t.palette[0] = new_color;
                                        }
                                    }
                                });
                        })
                    />

                    // Text Color
                    <ColorPicker
                        label="Text".to_string()
                        color=Signal::derive(move || {
                            local_config
                                .get()
                                .theme
                                .as_ref()
                                .map(|t| t.text_color.clone())
                                .unwrap_or("#333333".to_string())
                        })
                        on_change=Callback::new(move |new_color| {
                            local_config
                                .update(|c| {
                                    if c.theme.is_none() {
                                        c.theme = Some(ChartTheme::default());
                                    }
                                    if let Some(t) = c.theme.as_mut() {
                                        t.text_color = new_color;
                                    }
                                });
                        })
                    />

                    // Grid Color
                    <ColorPicker
                        label="Grid Lines".to_string()
                        color=Signal::derive(move || {
                            local_config
                                .get()
                                .grid
                                .as_ref()
                                .map(|g| g.color.clone())
                                .or_else(|| {
                                    local_config.get().theme.as_ref().map(|t| t.grid.color.clone())
                                })
                                .unwrap_or("#e0e0e0".to_string())
                        })
                        on_change=Callback::new(move |new_color| {
                            local_config
                                .update(|c| {
                                    let g = c.grid.get_or_insert_with(GridStyle::default);
                                    g.color = new_color;
                                });
                        })
                    />

                    // Axis Color
                    <ColorPicker
                        label="Axis Lines".to_string()
                        color=Signal::derive(move || {
                            local_config
                                .get()
                                .theme
                                .as_ref()
                                .map(|t| t.axis_color.clone())
                                .unwrap_or("#6E7079".to_string())
                        })
                        on_change=Callback::new(move |new_color| {
                            local_config
                                .update(|c| {
                                    if c.theme.is_none() {
                                        c.theme = Some(ChartTheme::default());
                                    }
                                    if let Some(t) = c.theme.as_mut() {
                                        t.axis_color = new_color;
                                    }
                                });
                        })
                    />
                </div>

                <div class="divider my-2"></div>
                <button class="btn btn-primary btn-sm w-full" on:click=on_save>
                    "Save Changes"
                </button>
            </div>
        </div>
    }
}
