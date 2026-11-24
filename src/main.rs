use leptos::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen::JsCast;

const K: [&str; 10] = ["#40566B", "#47604F", "#536A42", "#8A8F38", "#E1B72D", "#E5992A", "#E46B24", "#E12C2C", "#C32429", "#A51F26"];
const NM: [&str; 5] = ["LOW KEY", "STEADY", "IN MOTION", "HIGH ENERGY", "PEAK"];
const RX: [(&str, &str); 6] = [("♥", "Feel you"), ("🤝", "I’m here"), ("⚡", "Energy"), ("💬", "Let’s talk"), ("🎉", "Celebrate"), ("≈", "Same pulse")];
const USERS: [(&str, &str, &str); 6] = [("vale", "Valeria", "🎸"), ("mateo", "Mateo", "💻"), ("cami", "Camila", "🌵"), ("andres", "Andrés", "⚽"), ("lu", "Lucía", "🍳"), ("juli", "Julián", "📚")];
const GROUPS: [(&str, &str, &str); 3] = [("🎵", "Música", "group"), ("💻", "Código", "forum"), ("📣", "Avisos", "broadcast")];
const EMO: [&str; 6] = ["🙂", "😎", "🦊", "🌵", "🎧", "🚀"];
const REPLIES: [&str; 4] = ["Te leo 👀", "Aquí estoy 🤝", "Cuéntame más", "¡Qué bien!"];
const FRIENDS: [&str; 2] = ["vale", "cami"];

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Post {
    id: u64,
    user: String,
    score: u8,
    text: String,
    counts: [u32; 6],
    mine: [bool; 6],
    comments: Vec<(String, String)>,
    ts: f64,
}

#[derive(Clone, Copy)]
struct Ctx {
    posts: RwSignal<Vec<Post>>,
    me: RwSignal<String>,
    emo: RwSignal<&'static str>,
    col: RwSignal<&'static str>,
    open: RwSignal<u64>,
    tab: RwSignal<u8>,
    sheet: RwSignal<bool>,
    filt: RwSignal<u8>,
    spin: RwSignal<bool>,
    score: RwSignal<u8>,
    text: RwSignal<String>,
    grp: RwSignal<Option<usize>>,
    gmsg: RwSignal<Vec<(usize, String, String)>>,
    chat: RwSignal<Option<usize>>,
    dms: RwSignal<Vec<(usize, bool, String)>>,
}

fn band(s: u8) -> usize {
    ((s.max(1) - 1) / 2) as usize
}
fn name_of(id: &str) -> String {
    USERS.iter().find(|u| u.0 == id).map(|u| u.1.to_string()).unwrap_or_else(|| id.to_string())
}
fn emoji_of(id: &str) -> &'static str {
    USERS.iter().find(|u| u.0 == id).map(|u| u.2).unwrap_or("🙂")
}
fn store() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}
fn load() -> Option<Vec<Post>> {
    serde_json::from_str(&store()?.get_item("pulse.rs").ok()??).ok()
}
fn save(p: &Vec<Post>) {
    if let (Some(s), Ok(j)) = (store(), serde_json::to_string(p)) {
        let _ = s.set_item("pulse.rs", &j);
    }
}
fn seed() -> Vec<Post> {
    let now = js_sys::Date::now();
    let p = |id: u64, u: &str, s: u8, t: &str, c: [u32; 6]| Post {
        id, user: u.into(), score: s, text: t.into(), counts: c, mine: [false; 6], comments: vec![], ts: now - id as f64 * 3.6e6,
    };
    vec![
        p(1, "vale", 7, "Ensayo largo con la banda: salimos sin voz, pero con ganas de repetir.", [6, 0, 9, 0, 5, 0]),
        p(2, "mateo", 3, "Día tranquilo de refactorizar. Café y silencio.", [4, 7, 0, 0, 0, 0]),
        p(3, "cami", 9, "Hoy fue intenso de principio a fin: presentación, viaje y cena familiar.", [11, 0, 0, 8, 0, 0]),
        p(4, "andres", 5, "Partido de los miércoles. Perdimos, pero nos reímos muchísimo.", [0, 0, 12, 0, 3, 0]),
        p(5, "lu", 0, "Probé arepas rellenas. ¿Algún truco para que no se rompan?", [0, 3, 0, 9, 0, 0]),
        p(6, "juli", 2, "Semana de exámenes. Hoy avancé un poco, y está bien.", [14, 10, 0, 0, 0, 7]),
    ]
}
fn arc(i: usize) -> String {
    let pt = |d: f64| {
        let r = d.to_radians();
        (110.0 + 84.0 * r.cos(), 110.0 + 84.0 * r.sin())
    };
    let a = pt(i as f64 * 36.0 - 88.0);
    let b = pt((i as f64 + 1.0) * 36.0 - 92.0);
    format!("M{} {}A84 84 0 0 1 {} {}", a.0, a.1, b.0, b.1)
}
fn badge(s: u8) -> impl IntoView {
    view! { <span class="bg" style=format!("--k:{}", K[s as usize - 1])><i></i><b>{s}</b>{NM[band(s)]}</span> }
}
fn open_sheet(c: Ctx) {
    c.score.set(0);
    c.sheet.set(true);
}
fn publish(c: Ctx) {
    let s = c.score.get_untracked();
    let t = c.text.get_untracked();
    if s == 0 && t.trim().is_empty() {
        return;
    }
    let n = js_sys::Date::now();
    c.posts.update(|v| v.push(Post { id: n as u64, user: "me".into(), score: s, text: t.trim().into(), counts: [0; 6], mine: [false; 6], comments: vec![], ts: n }));
    c.text.set(String::new());
    c.sheet.set(false);
    c.tab.set(0);
    c.filt.set(0);
}
fn refresh(c: Ctx) {
    if c.spin.get_untracked() {
        return;
    }
    c.spin.set(true);
    set_timeout(
        move || {
            let pool = [("lu", 6u8, "Salió la receta, huele increíble."), ("andres", 8, "Gol en el último minuto."), ("juli", 3, "Repaso tranquilo y música baja."), ("cami", 5, "Entre clases y mandados, avanzando.")];
            let t = js_sys::Date::now();
            let k = pool[(t as usize) % 4];
            c.posts.update(|v| v.push(Post { id: t as u64, user: k.0.into(), score: k.1, text: k.2.into(), counts: [0; 6], mine: [false; 6], comments: vec![], ts: t }));
            c.spin.set(false);
            c.tab.set(0);
        },
        Duration::from_millis(800),
    );
}

#[component]
fn Wheel(score: RwSignal<u8>) -> impl IntoView {
    let pick = move |ev: web_sys::PointerEvent| {
        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let r = el.get_bounding_client_rect();
            let dx = ev.client_x() as f64 - r.left() - r.width() / 2.0;
            let dy = ev.client_y() as f64 - r.top() - r.height() / 2.0;
            if dx.hypot(dy) < r.width() * 0.2 {
                return;
            }
            let g = ((dy.atan2(dx).to_degrees() + 90.0) % 360.0 + 360.0) % 360.0;
            score.set(((g / 36.0) as u8 + 1).min(10));
        }
    };
    view! {
        <svg id="wh" viewBox="0 0 220 220" role="slider" tabindex="0" aria-label="Intensidad del día de 1 a 10"
            aria-valuemin="1" aria-valuemax="10" aria-valuenow=move || score.get().to_string()
            on:pointerdown=pick
            on:pointermove=move |e| { if e.buttons() > 0 { pick(e) } }
            on:keydown=move |e: web_sys::KeyboardEvent| match e.key().as_str() {
                "ArrowRight" | "ArrowUp" => score.update(|s| *s = (*s + 1).min(10)),
                "ArrowLeft" | "ArrowDown" => score.update(|s| *s = s.saturating_sub(1).max(1)),
                _ => {}
            }>
            {(0..10).map(|i| view! {
                <path class=move || if score.get() as usize == i + 1 { "sg on" } else { "sg" } d=arc(i) stroke=K[i]/>
            }).collect_view()}
            <text x="110" y="124" class="mo"
                style=move || format!("font-size:46px;fill:{}", if score.get() > 0 { K[score.get() as usize - 1] } else { "var(--mu)" })>
                {move || if score.get() > 0 { score.get().to_string() } else { "–".to_string() }}
            </text>
            <text x="110" y="146" style="font-size:11px;letter-spacing:.06em">
                {move || if score.get() > 0 { NM[band(score.get())].to_string() } else { "OPCIONAL".to_string() }}
            </text>
        </svg>
    }
}

fn card(c: Ctx, p: Post) -> View {
    let id = p.id;
    let own = p.user == "me";
    let (name, em) = if own { (c.me.get_untracked(), c.emo.get_untracked()) } else { (name_of(&p.user), emoji_of(&p.user)) };
    let bg = if own { c.col.get_untracked() } else { "#536A42" };
    let cm = create_rw_signal(String::new());
    let comments = p.comments.clone();
    let ncm = comments.len();
    let react = RX.iter().enumerate().map(|(i, r)| {
        let on = p.mine[i];
        let n = p.counts[i] + on as u32;
        view! {
            <button class="chip" aria-pressed=on.to_string() aria-label=r.1
                on:click=move |_| c.posts.update(|v| if let Some(x) = v.iter_mut().find(|x| x.id == id) { x.mine[i] = !x.mine[i] })>
                {r.0}{(n > 0).then(|| view! { <b class="mo">{n}</b> })}
            </button>
        }
    }).collect_view();
    view! {
        <article class="cd">
            <div class="row">
                <div class="av" style=format!("background:{bg}")>{em}</div>
                <div class="gr"><strong>{name}</strong></div>
                {(p.score > 0).then(|| badge(p.score))}
                {own.then(|| view! { <button class="ln" aria-label="Eliminar" on:click=move |_| c.posts.update(|v| v.retain(|x| x.id != id))>"🗑"</button> })}
            </div>
            {(!p.text.is_empty()).then(|| view! { <p>{p.text.clone()}</p> })}
            <div class="rx">
                {react}
                <button class="chip" aria-label="Comentarios" on:click=move |_| c.open.update(|o| *o = if *o == id { 0 } else { id })>"💭"{ncm}</button>
            </div>
            {move || (c.open.get() == id).then(|| {
                let comments = comments.clone();
                view! {
                    <div class="th">
                        {comments.into_iter().map(|(u, t)| view! { <p><strong>{u}</strong>" "{t}</p> }).collect_view()}
                        <div class="row">
                            <input placeholder="Comenta…" aria-label="Comentario" prop:value=move || cm.get() on:input=move |e| cm.set(event_target_value(&e))/>
                            <button class="chip" aria-label="Enviar" on:click=move |_| {
                                let t = cm.get();
                                if t.trim().is_empty() { return; }
                                let n = c.me.get_untracked();
                                c.posts.update(|v| if let Some(x) = v.iter_mut().find(|x| x.id == id) { x.comments.push((n, t)) });
                                cm.set(String::new());
                            }>"➤"</button>
                        </div>
                    </div>
                }
            })}
        </article>
    }.into_view()
}

fn feed(c: Ctx) -> View {
    let list = move || {
        let _ = (c.me.get(), c.emo.get(), c.col.get());
        let mut v = c.posts.get();
        v.sort_by(|a, b| b.ts.partial_cmp(&a.ts).unwrap());
        let my = v.iter().find(|p| p.user == "me" && p.score > 0).map(|p| p.score as i32).unwrap_or(0);
        match c.filt.get() {
            1 => v.retain(|p| p.user == "me" || FRIENDS.contains(&p.user.as_str())),
            2 => v.retain(|p| p.user != "me" && p.score > 0 && my > 0 && (p.score as i32 - my).abs() <= 1),
            3 => v.sort_by_key(|p| std::cmp::Reverse(p.counts.iter().sum::<u32>())),
            _ => {}
        }
        v
    };
    view! {
        <div class="fl">
            {[(0u8, "★ Para ti"), (1, "👥 Amigos"), (2, "≈ Mismo pulso"), (3, "🔥 Top")].into_iter().map(|(i, t)| view! {
                <button class="chip" aria-pressed=move || (c.filt.get() == i).to_string() on:click=move |_| c.filt.set(i)>{t}</button>
            }).collect_view()}
        </div>
        {move || list().into_iter().map(|p| card(c, p)).collect_view()}
    }.into_view()
}

fn groups(c: Ctx) -> View {
    view! {
        {move || match c.grp.get() {
            None => view! {
                <h2>"Grupos"</h2>
                {GROUPS.iter().enumerate().map(|(i, g)| view! {
                    <div class="cd row">
                        <div class="av" style="background:var(--bg)">{g.0}</div>
                        <div class="gr"><strong>{g.1}</strong><small>{match g.2 { "forum" => "Foro", "broadcast" => "Difusión", _ => "Grupo" }}</small></div>
                        <button class="chip" on:click=move |_| c.grp.set(Some(i))>"Abrir"</button>
                    </div>
                }).collect_view()}
            }.into_view(),
            Some(i) => group_chat(c, i),
        }}
    }.into_view()
}

fn group_chat(c: Ctx, i: usize) -> View {
    let g = GROUPS[i];
    let t = create_rw_signal(String::new());
    view! {
        <div class="row"><button class="chip" aria-label="Volver" on:click=move |_| c.grp.set(None)>"←"</button><strong class="gr">{g.0}" "{g.1}</strong></div>
        {move || c.gmsg.get().into_iter().filter(|m| m.0 == i).map(|m| view! { <div class="ms"><b>{m.1}</b><p>{m.2}</p></div> }).collect_view()}
        {if g.2 != "broadcast" {
            view! {
                <div class="row">
                    <input placeholder="Escribe…" aria-label="Mensaje" prop:value=move || t.get() on:input=move |e| t.set(event_target_value(&e))/>
                    <button class="chip on" aria-label="Enviar" on:click=move |_| {
                        let x = t.get();
                        if x.trim().is_empty() { return; }
                        let n = c.me.get_untracked();
                        c.gmsg.update(|v| v.push((i, n, x)));
                        t.set(String::new());
                    }>"➤"</button>
                </div>
            }.into_view()
        } else {
            view! { <small>"📣 Solo el administrador publica."</small> }.into_view()
        }}
    }.into_view()
}

fn chats(c: Ctx) -> View {
    view! {
        {move || match c.chat.get() {
            None => view! {
                <h2>"Mensajes"</h2>
                {USERS.iter().enumerate().map(|(i, u)| view! {
                    <div class="cd row">
                        <div class="av" style="background:var(--bg)">{u.2}</div>
                        <div class="gr"><strong>{u.1}</strong></div>
                        <button class="chip" aria-label="Abrir chat" on:click=move |_| c.chat.set(Some(i))>"✉"</button>
                    </div>
                }).collect_view()}
            }.into_view(),
            Some(i) => chat(c, i),
        }}
    }.into_view()
}

fn chat(c: Ctx, i: usize) -> View {
    let t = create_rw_signal(String::new());
    let send = move || {
        let x = t.get_untracked();
        if x.trim().is_empty() { return; }
        c.dms.update(|v| v.push((i, true, x)));
        t.set(String::new());
        set_timeout(
            move || c.dms.update(|v| v.push((i, false, REPLIES[(js_sys::Date::now() as usize) % 4].to_string()))),
            Duration::from_millis(1200),
        );
    };
    view! {
        <div class="row"><button class="chip" aria-label="Volver" on:click=move |_| c.chat.set(None)>"←"</button><strong class="gr">{USERS[i].2}" "{USERS[i].1}</strong></div>
        {move || c.dms.get().into_iter().filter(|m| m.0 == i).map(|m| view! { <div class=if m.1 { "ms me" } else { "ms" }>{m.2}</div> }).collect_view()}
        <div class="row">
            <input placeholder="Mensaje" aria-label="Mensaje" prop:value=move || t.get() on:input=move |e| t.set(event_target_value(&e))
                on:keydown=move |e: web_sys::KeyboardEvent| if e.key() == "Enter" { send() }/>
            <button class="chip on" aria-label="Enviar" on:click=move |_| send()>"➤"</button>
        </div>
    }.into_view()
}

fn me(c: Ctx) -> View {
    view! {
        <div class="cd">
            <div class="bn" style=move || format!("background:{}", c.col.get())></div>
            <div class="av lg2" style=move || format!("background:{}", c.col.get())>{move || c.emo.get()}</div>
            <input aria-label="Nombre" prop:value=move || c.me.get() on:input=move |e| c.me.set(event_target_value(&e))/>
            <div class="fl">{EMO.iter().map(|e| { let e = *e; view! {
                <button class="chip" aria-pressed=move || (c.emo.get() == e).to_string() on:click=move |_| c.emo.set(e)>{e}</button>
            }}).collect_view()}</div>
            <div class="fl">{K.iter().map(|k| { let k = *k; view! {
                <button class="chip" style=format!("background:{k};border-color:{k};color:#fff") aria-label="Color del perfil" on:click=move |_| c.col.set(k)>
                    {move || if c.col.get() == k { "✓" } else { "" }}
                </button>
            }}).collect_view()}</div>
        </div>
    }.into_view()
}

fn nb(c: Ctx, i: u8, s: &'static str, l: &'static str) -> impl IntoView {
    view! {
        <button aria-label=l aria-current=move || if c.tab.get() == i { "page" } else { "false" } on:click=move |_| c.tab.set(i)>{s}</button>
    }
}

#[component]
fn App() -> impl IntoView {
    let init = load().unwrap_or_else(seed);
    let now = js_sys::Date::now();
    let has_today = init.iter().any(|p| p.user == "me" && p.score > 0 && now - p.ts < 8.64e7);
    let c = Ctx {
        posts: create_rw_signal(init),
        me: create_rw_signal("Tú".to_string()),
        emo: create_rw_signal(EMO[0]),
        col: create_rw_signal(K[0]),
        open: create_rw_signal(0),
        tab: create_rw_signal(0),
        sheet: create_rw_signal(!has_today),
        filt: create_rw_signal(0),
        spin: create_rw_signal(false),
        score: create_rw_signal(0),
        text: create_rw_signal(String::new()),
        grp: create_rw_signal(None),
        gmsg: create_rw_signal(vec![
            (0, "Valeria".into(), "¿Quién va al concierto del sábado?".into()),
            (0, "Mateo".into(), "Yo llevo la bocina 🔊".into()),
            (1, "Mateo".into(), "¿Vanilla JS o framework para un MVP?".into()),
            (2, "Valeria".into(), "Mañana publicamos el calendario de ensayos.".into()),
        ]),
        chat: create_rw_signal(None),
        dms: create_rw_signal(vec![]),
    };
    create_effect(move |_| save(&c.posts.get()));
    view! {
        <div class="w">
            <header class="top">
                <div class="lg">"PULSE"<i>"●"</i></div>
                <button class=move || if c.spin.get() { "chip spin" } else { "chip" } aria-label="Recargar feed" on:click=move |_| refresh(c)>"↻"</button>
            </header>
            <main>{move || match c.tab.get() { 0 => feed(c), 1 => groups(c), 2 => chats(c), _ => me(c) }}</main>
        </div>
        <nav class="nv" aria-label="Principal">
            {nb(c, 0, "⌂", "Feed")}{nb(c, 1, "▦", "Grupos")}
            <button class="plus" aria-label="Registrar Daily Pulse" on:click=move |_| open_sheet(c)>"＋"</button>
            {nb(c, 2, "✉", "Mensajes")}{nb(c, 3, "☻", "Perfil")}
        </nav>
        {move || c.sheet.get().then(|| view! {
            <div class="ov" on:click=move |_| c.sheet.set(false)>
                <div class="sh up" role="dialog" aria-modal="true" aria-label="Daily Pulse" on:click=|e| e.stop_propagation()>
                    <h2 style="text-align:center">"How was your day?"</h2>
                    <Wheel score=c.score/>
                    <p class="mu" style="text-align:center">"Gira la rueda. El color marca intensidad, no si fue bueno o malo."</p>
                    <textarea placeholder="¿Algo que contar?" aria-label="Texto" prop:value=move || c.text.get() on:input=move |e| c.text.set(event_target_value(&e))></textarea>
                    <div class="row">
                        <button class="chip" aria-label="Omitir" on:click=move |_| c.sheet.set(false)>"✕"</button>
                        <button class="chip on gr" on:click=move |_| publish(c)>"Publicar ✓"</button>
                    </div>
                </div>
            </div>
        })}
    }
}

fn main() {
    mount_to_body(App)
}
