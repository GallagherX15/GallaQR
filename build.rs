fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/gallaqr.ico");
        res.set("ProductName", "GallaQR");
        res.set("FileDescription", "GallaQR - Generador de Códigos QR");
        res.set("LegalCopyright", "GallaQR 2026");
        let _ = res.compile();
    }
}

