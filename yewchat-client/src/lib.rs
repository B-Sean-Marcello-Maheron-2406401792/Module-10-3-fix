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
}

#[derive(PartialEq)]
enum Screen { Login, Chat }

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
}

const AVATARS: &[&str] = &[
    "🦊","🐺","🐧","🦁","🐸","🤖","👾","🦄","🐙","🦋","🐉","👻",
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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetName(v)   => { self.name = v; true }
            Msg::SetServer(v) => { self.server_url = v; true }
            Msg::SetAvatar(v) => { self.selected_avatar = v; true }

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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.screen {
            Screen::Login => self.view_login(ctx),
            Screen::Chat  => self.view_chat(ctx),
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
                    <p>{ "// Rust + Yew + WebSocket" }</p>
                </div>
                <div class="avatar-picker">
                    <div class="avatar-label">{ "// SELECT AVATAR" }</div>
                    <div class="avatar-grid">{ avatars }</div>
                </div>
                <div class="login-form">
                    <div>
                        <div class="field-label">{ "// CALLSIGN" }</div>
                        <input class="neon-input" type="text"
                            placeholder="e.g. Ade's Komputer"
                            value={self.name.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                Msg::SetName(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                            })}
                            // PERBAIKAN: Menggunakan batch_callback agar tidak menghapus input saat mengetik
                            onkeydown={link.batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" { Some(Msg::Connect) }
                                else { None }
                            })}
                        />
                    </div>
                    <div>
                        <div class="field-label">{ "// SERVER URL" }</div>
                        <input class="neon-input" type="text"
                            value={self.server_url.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                Msg::SetServer(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                            })}
                        />
                    </div>
                    if self.login_error {
                        <span class="login-error">
                            { "⚠ CONNECTION REFUSED — IS SERVER RUNNING?" }
                        </span>
                    }
                    <button class="btn-connect"
                        onclick={link.callback(|_| Msg::Connect)}>
                        { "// CONNECT" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_chat(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let status_class = if self.connected { "status live" } else { "status" };
        let status_text  = if self.connected { "● CONNECTED" } else { "○ OFFLINE" };

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
                        <span class="brand-sub">{ "Rust · Yew · WebSocket" }</span>
                    </div>
                    <span class={status_class}>{ status_text }</span>
                    <div class="header-right">
                        <span class="user-pill">
                            { &self.selected_avatar }{ " " }{ &self.name }
                        </span>
                        <button class="btn-dc"
                            onclick={link.callback(|_| Msg::Disconnect)}>
                            { "DISCONNECT" }
                        </button>
                    </div>
                </div>

                <div class="messages">{ msgs }</div>

                <div class="input-area">
                    <span class="prompt">{ ">_" }</span>
                    <input class="msg-input" type="text"
                        placeholder="transmit message..."
                        value={self.input.clone()}
                        disabled={!self.connected}
                        oninput={link.callback(|e: InputEvent| {
                            Msg::SetInput(e.target_unchecked_into::<web_sys::HtmlInputElement>().value())
                        })}
                        // PERBAIKAN: Menggunakan batch_callback agar tidak menghapus pesan saat mengetik
                        onkeydown={link.batch_callback(|e: KeyboardEvent| {
                            if e.key() == "Enter" { Some(Msg::SendMessage) }
                            else { None }
                        })}
                    />
                    <button class="btn-send" disabled={!self.connected}
                        onclick={link.callback(|_| Msg::SendMessage)}>
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