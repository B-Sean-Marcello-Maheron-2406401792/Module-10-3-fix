use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub sender: String,
    pub content: String,
    pub avatar: Option<String>,
    pub addr: Option<String>,
    pub timestamp: Option<String>,
}

pub struct App {
    screen: Screen,
    name: String,
    server_url: String,
    selected_avatar: String,
    login_error: bool,
    ws: Option<WebSocket>,
    messages: Vec<ChatMessage>,
    input: String,
    connected: bool,
    emoji_picker_open: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Screen { Login, Chat, About, Gallery }

pub enum Msg {
    SetName(String),
    SetServer(String),
    SetAvatar(String),
    Connect,
    WsOpen,
    WsMessage(String),
    WsError,
    WsClose,
    SetInput(String),
    SendMessage,
    Disconnect,
    SwitchScreen(Screen),
    ToggleEmojiPicker,
    AddEmoji(String),
}

const AVATARS: &[&str] = &[
    "🦊","🐺","🐧","🦁","🐸","🤖","👾","🦄","🐙","🦋","🐉","👻",
    "🚀","🛸","🌌","🛰️","🌠","☄️","⚡","🔥","❄️","🍀","💎","🧿",
    "🎨","🎭","🎪","🎢","🎡","🎬","🎤","🎧","🎷","🎸","🎹","🎺",
];

const EMOJIS: &[&str] = &[
    "😀","😃","😄","😁","😆","😅","😂","🤣","😊","😇","🙂","🙃",
    "😉","😌","😍","🥰","😘","😗","😙","😚","😋","😛","😝","😜",
    "🤪","🤨","🧐","🤓","😎","🤩","🥳","😏","😒","😞","😔","😟",
    "😕","🙁","☹️","😣","😖","😫","😩","🥺","😢","😭","😤","😠",
    "😡","🤬","🤯","😳","🥵","🥶","😱","😨","😰","😥","😓","🤗",
    "🤔","🤭","🤫","🤥","😶","😐","😑","😬","🙄","😯","😦","😧",
    "😮","😲","🥱","😴","🤤","😪","😵","🤐","🥴","🤢","🤮","🤧",
    "😷","🤒","🤕","🤑","🤠","😈","👿","👹","👺","🤡","👻","💀",
    "☠️","👽","👾","🤖","🎃","😺","😸","😹","😻","😼","😽","🙀",
    "😿","😾","👋","🤚","🖐️","✋","🖖","👌","🤏","✌️","🤞","🤟",
    "🤘","🤙","👈","👉","👆","🖕","👇","☝️","👍","👎","✊","👊",
    "🤛","🤜","👏","🙌","👐","🤲","🤝","🙏","✍️","💅","🤳","💪",
    "🦾","🦵","🦿","🦶","👂","🦻","👃","🧠","🦷","🦴","👀","👁️",
    "👅","👄","💋","🩸","❤️","🧡","💛","💚","💙","💜","🖤","🤍",
    "🤎","💔","🔥","✨","🌟","💫","💥","💢","💦","💨","🕳️","💣",
];

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            screen: Screen::Login,
            name: String::new(),
            server_url: "ws://127.0.0.1:9001".into(),
            selected_avatar: "🦊".into(),
            login_error: false,
            ws: None,
            messages: Vec::new(),
            input: String::new(),
            connected: false,
            emoji_picker_open: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetName(v)   => { self.name = v; true }
            Msg::SetServer(v) => { self.server_url = v; true }
            Msg::SetAvatar(v) => { self.selected_avatar = v; true }
            Msg::SwitchScreen(s) => { self.screen = s; true }

            Msg::Connect => {
                if self.name.trim().is_empty() { return false; }
                match WebSocket::new(&self.server_url) {
                    Ok(ws) => {
                        let link = ctx.link().clone();
                        let cb = Closure::<dyn Fn()>::wrap(Box::new(move || {
                            link.send_message(Msg::WsOpen);
                        }));
                        ws.set_onopen(Some(cb.as_ref().unchecked_ref()));
                        cb.forget();

                        let link = ctx.link().clone();
                        // PERBAIKAN: Menambahkan tipe data eksplisit `: MessageEvent`
                        let cb = Closure::<dyn Fn(MessageEvent)>::wrap(Box::new(move |e: MessageEvent| {
                            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                                link.send_message(Msg::WsMessage(txt.into()));
                            }
                        }));
                        ws.set_onmessage(Some(cb.as_ref().unchecked_ref()));
                        cb.forget();

                        let link = ctx.link().clone();
                        let cb = Closure::<dyn Fn(ErrorEvent)>::wrap(Box::new(move |_| {
                            link.send_message(Msg::WsError);
                        }));
                        ws.set_onerror(Some(cb.as_ref().unchecked_ref()));
                        cb.forget();

                        let link = ctx.link().clone();
                        let cb = Closure::<dyn Fn(CloseEvent)>::wrap(Box::new(move |_| {
                            link.send_message(Msg::WsClose);
                        }));
                        ws.set_onclose(Some(cb.as_ref().unchecked_ref()));
                        cb.forget();

                        self.ws = Some(ws);
                        self.login_error = false;
                        true
                    }
                    Err(_) => { self.login_error = true; true }
                }
            }

            Msg::WsOpen => {
                self.connected = true;
                self.screen = Screen::Chat;
                if let Some(ws) = &self.ws {
                    let join = ChatMessage {
                        msg_type: "join".into(),
                        sender: self.name.clone(),
                        content: String::new(),
                        avatar: Some(self.selected_avatar.clone()),
                        addr: None,
                        timestamp: None,
                    };
                    let _ = ws.send_with_str(&serde_json::to_string(&join).unwrap());
                }
                true
            }

            Msg::WsMessage(raw) => {
                if let Ok(msg) = serde_json::from_str::<ChatMessage>(&raw) {
                    if msg.msg_type != "join" {
                        self.messages.push(msg);
                    }
                }
                true
            }

            Msg::WsError => {
                self.login_error = true;
                self.ws = None;
                self.screen = Screen::Login;
                true
            }

            Msg::WsClose => {
                self.connected = false;
                self.messages.push(ChatMessage {
                    msg_type: "system".into(),
                    sender: "Server".into(),
                    content: "Connection closed.".into(),
                    avatar: None, addr: None, timestamp: None,
                });
                true
            }

            Msg::SetInput(v) => { self.input = v; true }

            Msg::SendMessage => {
                let content = self.input.trim().to_string();
                if content.is_empty() { return false; }
                if let Some(ws) = &self.ws {
                    let msg = ChatMessage {
                        msg_type: "chat".into(),
                        sender: self.name.clone(),
                        content,
                        avatar: Some(self.selected_avatar.clone()),
                        addr: None,
                        timestamp: None,
                    };
                    let _ = ws.send_with_str(&serde_json::to_string(&msg).unwrap());
                    self.input.clear();
                    self.emoji_picker_open = false;
                }
                true
            }

            Msg::Disconnect => {
                if let Some(ws) = &self.ws { let _ = ws.close(); }
                self.ws = None;
                self.connected = false;
                self.screen = Screen::Login;
                self.messages.clear();
                true
            }

            Msg::ToggleEmojiPicker => {
                self.emoji_picker_open = !self.emoji_picker_open;
                true
            }

            Msg::AddEmoji(e) => {
                self.input.push_str(&e);
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let window = web_sys::window().unwrap();
        if let Ok(lucide) = js_sys::Reflect::get(&window, &JsValue::from_str("lucide")) {
            if !lucide.is_undefined() && !lucide.is_null() {
                if let Ok(create_icons) = js_sys::Reflect::get(&lucide, &JsValue::from_str("createIcons")) {
                    if create_icons.is_function() {
                        let _ = js_sys::Reflect::apply(&create_icons.into(), &lucide, &js_sys::Array::new());
                    }
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.screen {
            Screen::Login   => self.view_login(ctx),
            Screen::Chat    => self.view_chat(ctx),
            Screen::About   => self.view_about(ctx),
            Screen::Gallery => self.view_gallery(ctx),
        }
    }
}

impl App {
    fn view_login(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let avatars: Html = AVATARS.iter().map(|&av| {
            let av_s = av.to_string();
            let selected = av_s == self.selected_avatar;
            let cb = link.callback(move |_| Msg::SetAvatar(av_s.clone()));
            html! {
                <button class={if selected {"avatar-opt selected"} else {"avatar-opt"}}
                    onclick={cb}>{ av }</button>
            }
        }).collect();

        html! {
            <div id="login">
                <div class="login-header">
                    <h1>{ "YEWCHAT" }</h1>
                    <p>{ "// REWIRING REALITY WITH RUST" }</p>
                    <div class="header-links">
                        <button class="link-btn" onclick={link.callback(|_| Msg::SwitchScreen(Screen::About))}>
                            <i data-lucide="info"></i>
                            { "SYSTEM INTEL" }
                        </button>
                        <button class="link-btn" onclick={link.callback(|_| Msg::SwitchScreen(Screen::Gallery))}>
                            <i data-lucide="palette"></i>
                            { "ART GALLERY" }
                        </button>
                    </div>
                </div>
                <div class="avatar-picker">
                    <div class="avatar-label">{ "// SELECT YOUR AVATAR" }</div>
                    <div class="avatar-grid">{ avatars }</div>
                </div>
                <div class="login-form">
                    <div>
                        <div class="field-label">{ "// CALLSIGN" }</div>
                        <input class="neon-input" type="text"
                            placeholder="e.g. Cyber_Nomad"
                            value={self.name.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                Msg::SetName(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                            })}
                            onkeydown={link.batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" { Some(Msg::Connect) }
                                else { None }
                            })}
                        />
                    </div>
                    <div>
                        <div class="field-label">{ "// UPLINK URL" }</div>
                        <input class="neon-input" type="text"
                            value={self.server_url.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                Msg::SetServer(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                            })}
                        />
                    </div>
                    if self.login_error {
                        <span class="login-error">
                            { "⚠ UPLINK FAILURE — VERIFY SERVER STATUS" }
                        </span>
                    }
                    <button class="btn-connect"
                        onclick={link.callback(|_| Msg::Connect)}>
                        <i data-lucide="zap"></i>
                        { "INITIATE CONNECTION" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_about(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div id="about-screen">
                <div class="about-container">
                    <div class="about-header">
                        <h2><i data-lucide="database"></i>{ "SYSTEM INTEL" }</h2>
                        <div class="glitch-line"></div>
                    </div>
                    <div class="about-content">
                        <section>
                            <h3>{ "> PROJECT_CORE" }</h3>
                            <p>{ "YewChat is a decentralized communication terminal designed for the next era of the web. Built with Rust and compiled to WebAssembly, it ensures safety, speed, and creative freedom." }</p>
                        </section>
                        <section>
                            <h3>{ "> TECHNICAL_STACK" }</h3>
                            <ul class="tech-list">
                                <li><span>{ "ENGINE:" }</span>{ " Rust (The language of the future)" }</li>
                                <li><span>{ "INTERFACE:" }</span>{ " Yew Framework (Blazing fast UI)" }</li>
                                <li><span>{ "TRANSPORT:" }</span>{ " WebSockets (Real-time pulses)" }</li>
                                <li><span>{ "COMPILED:" }</span>{ " WebAssembly (Near-native speed)" }</li>
                            </ul>
                        </section>
                        <section>
                            <h3>{ "> THE_VISION" }</h3>
                            <p>{ "We believe that software should not only be functional but also a work of art. YewChat is our canvas, and Rust is our brush." }</p>
                            <div class="creativity-quote">
                                { "“The only way to predict the future is to build it ourselves, one line of code at a time.”" }
                                <br/>
                                <span class="quote-source">{ "— ANONYMOUS CODER" }</span>
                            </div>
                        </section>
                    </div>
                    <button class="btn-back" onclick={link.callback(|_| Msg::SwitchScreen(Screen::Login))}>
                        <i data-lucide="arrow-left"></i>
                        { "RETURN TO UPLINK" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_gallery(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let art_items = vec![
            ("🎨", "Digital Dreams"), ("🚀", "Space Explorer"), ("🤖", "AI Synthesis"),
            ("🌌", "Nebula Pulse"), ("🧬", "Genetic Code"), ("⚡", "High Voltage"),
            ("💎", "Data Crystal"), ("🧿", "Omni Sight"), ("🌈", "Spectrum Shift"),
            ("🛸", "Unidentified"), ("🐉", "Mythic Flow"), ("👾", "Glitch Spirit"),
        ];

        let gallery: Html = art_items.into_iter().map(|(icon, label)| {
            html! {
                <div class="art-item" data-label={label}>{ icon }</div>
            }
        }).collect();

        html! {
            <div id="gallery-screen">
                <div class="gallery-container">
                    <div class="gallery-header">
                        <h2><i data-lucide="palette"></i>{ "CREATIVE REPOSITORY" }</h2>
                        <div class="glitch-line"></div>
                    </div>
                    <div class="gallery-content">
                        <section>
                            <h3>{ "> IMAGINATION_LOGS" }</h3>
                            <p>{ "Explore the visual artifacts generated by the YewChat creativity engine. Each icon represents a fragment of a larger digital consciousness." }</p>
                        </section>
                        <div class="art-grid">
                            { gallery }
                        </div>
                        <section style="margin-top: 30px;">
                            <div class="creativity-quote" style="border-left-color: var(--neon2); color: var(--neon2); background: rgba(0, 207, 255, 0.05);">
                                { "“Logic will get you from A to B. Imagination will take you everywhere.”" }
                                <br/>
                                <span class="quote-source">{ "— ALBERT EINSTEIN" }</span>
                            </div>
                        </section>
                    </div>
                    <button class="btn-back" onclick={link.callback(|_| Msg::SwitchScreen(Screen::Login))}>
                        <i data-lucide="arrow-left"></i>
                        { "RETURN TO UPLINK" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_chat(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let status_class = if self.connected { "status live" } else { "status" };
        let status_text  = if self.connected { "● UPLINK LIVE" } else { "○ UPLINK DOWN" };

        let msgs: Html = self.messages.iter().map(|m| {
            let is_self = m.sender == self.name;
            let is_sys  = m.msg_type == "system";
            let cls = if is_sys { "msg sys" }
            else if is_self { "msg self" }
            else { "msg other" };
            let av = m.avatar.clone().unwrap_or("👤".into());
            let ts = m.timestamp.clone().unwrap_or_default();
            html! {
                <div class={cls}>
                    if !is_sys {
                        <div class="msg-meta">
                            <span>{ &av }</span>
                            <span>{ &m.sender }</span>
                            if let Some(a) = &m.addr {
                                <span class="addr">{ a }</span>
                            }
                        </div>
                    }
                    <div class="msg-bubble">{ &m.content }</div>
                    if !ts.is_empty() {
                        <div class="msg-time">{ &ts }</div>
                    }
                </div>
            }
        }).collect();

        html! {
            <div class="chat-screen">
                <div class="chat-header">
                    <div class="brand">
                        <span class="brand-name">{ "YEWCHAT" }</span>
                        <span class="brand-sub">{ "TRANSMISSION SECURE // 256-BIT" }</span>
                    </div>
                    <span class={status_class}>{ status_text }</span>
                    <div class="header-right">
                        <span class="user-pill">
                            { &self.selected_avatar }{ " " }{ &self.name }
                        </span>
                        <button class="btn-dc"
                            onclick={link.callback(|_| Msg::Disconnect)}>
                            <i data-lucide="log-out"></i>
                            { "TERMINATE" }
                        </button>
                    </div>
                </div>

                <div class="messages">{ msgs }</div>

                <div class="input-area">
                    <div class="emoji-area">
                        <button class="btn-emoji" onclick={link.callback(|_| Msg::ToggleEmojiPicker)}>
                            { "😀" }
                        </button>
                        if self.emoji_picker_open {
                            <div class="emoji-picker">
                                { for EMOJIS.iter().map(|&e| {
                                    let ev = e.to_string();
                                    html! {
                                        <button class="emoji-opt" onclick={link.callback(move |_| Msg::AddEmoji(ev.clone()))}>
                                            { e }
                                        </button>
                                    }
                                }) }
                            </div>
                        }
                    </div>
                    <span class="prompt">{ ">_" }</span>
                    <input class="msg-input" type="text"
                        placeholder="transmit neural pulse..."
                        value={self.input.clone()}
                        disabled={!self.connected}
                        oninput={link.callback(|e: InputEvent| {
                            Msg::SetInput(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                        })}
                        onkeydown={link.batch_callback(|e: KeyboardEvent| {
                            if e.key() == "Enter" { Some(Msg::SendMessage) }
                            else { None }
                        })}
                    />
                    <button class="btn-send" disabled={!self.connected}
                        onclick={link.callback(|_| Msg::SendMessage)}>
                        <i data-lucide="send"></i>
                        { "SEND" }
                    </button>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}