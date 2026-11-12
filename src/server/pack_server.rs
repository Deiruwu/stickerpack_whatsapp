use anyhow::anyhow;
use qrcode::QrCode;
use qrcode::render::unicode;

struct PackServer {
    zip: Vec<u8>,
    port: u16,
}

impl PackServer {
    fn new(zip: Vec<u8>, port: u16) -> Self {
        Self { zip, port }
    }

    fn url(&self) -> String {
        let host = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| {
                eprintln!("⚠ no pude detectar la IP local");
                "127.0.0.1".into()
            });
        format!("http://{host}:{}/pack.zip", self.port)
    }

    fn print_qr(url: &str) -> anyhow::Result<()> {
        let code = QrCode::new(url.as_bytes())?;
        // Invertido para terminales de fondo oscuro.
        let qr = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .quiet_zone(true)
            .build();
        println!("\n{qr}\n{url}\n(Ctrl+C para detener el servidor)");
        Ok(())
    }

    fn run(&self) -> anyhow::Result<()> {
        Self::print_qr(&self.url())?;

        let server = tiny_http::Server::http(("0.0.0.0", self.port))
            .map_err(|e| anyhow!("no pude abrir el puerto {}: {e}", self.port))?;
        for req in server.incoming_requests() {
            let resp = if req.url() == "/pack.zip" {
                let h = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/zip"[..])
                    .unwrap();
                req.respond(tiny_http::Response::from_data(self.zip.clone()).with_header(h))
            } else {
                req.respond(tiny_http::Response::from_string("404").with_status_code(404))
            };
            if let Err(e) = resp {
                eprintln!("error sirviendo petición: {e}");
            }
        }
        Ok(())
    }
}