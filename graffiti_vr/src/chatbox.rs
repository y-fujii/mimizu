use std::*;

pub struct ChatBox {
    input: String,
    typing: bool,
    time: time::Instant,
    socket: net::UdpSocket,
}

fn osc_str(buf: &mut Vec<u8>, s: &[u8]) {
    buf.extend(s);
    for _ in (buf.len() % 4)..4 {
        buf.push(0);
    }
}

impl ChatBox {
    pub fn new() -> io::Result<Self> {
        let socket = net::UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        socket.connect("127.0.0.1:9000")?;

        Ok(ChatBox {
            input: String::new(),
            typing: false,
            time: time::Instant::now(),
            socket: socket,
        })
    }

    pub fn input(&mut self, s: String) {
        self.input = s;
    }

    pub fn typing(&mut self, b: bool) {
        self.typing = b;
    }

    pub fn update(&mut self) {
        // VRChat rate limit.
        if self.time.elapsed() < time::Duration::from_millis(1250) {
            return;
        }

        let mut buf = Vec::new();
        osc_str(&mut buf, b"/chatbox/input");
        osc_str(&mut buf, b",sT");
        osc_str(&mut buf, self.input.as_bytes());
        self.socket.send(&buf).ok();

        buf.clear();
        osc_str(&mut buf, b"/chatbox/typing");
        osc_str(&mut buf, if self.typing { b",T" } else { b",F" });
        self.socket.send(&buf).ok();

        self.time = time::Instant::now();
    }
}
